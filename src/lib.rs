//this program can't use std since it's on bare metal
#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(allocator_api)]
#![feature(ptr_metadata)]
#![feature(const_mut_refs)]
#![feature(panic_info_message)]
#![feature(error_in_core)]
#![feature(result_flattening)]
#[macro_use]
extern crate bitflags;
extern crate alloc;

use alloc::boxed::Box;
use base::display::{DefaultVgaWriter, UniversalVgaFormatter, VgaColorCombo, VgaPalette, VgaColor};
use base::forth::{ForthMachine, StackItem};
use base::input::KEYBOARD_QUEUE;
use easter_eggs::SplashScreen;
use forth::Stack;

use base::*;
use fs::{FileSystemError, PathString};
use interrupt::setup::GLOBAL_OS_TIME;

use crate::interrupt::pitinit;

use crate::memory::populate_global_allocator;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use memory::paging::PageTableMaster;
use x86_64::instructions::port::PortWriteOnly;

pub mod cpuid;

mod easter_eggs;
mod panic;
use crate::multiboot_info::MultibootInfoHeader;

mod input;
mod interrupt;
mod memory;
mod multiboot_info;

// ``no_mangle`` säger till kompilatorn att bevara funktionens namn efter kompilering.
// detta tillåter long_mode.asm att tillkalla ``rust_start`` i slutet av dess kod vilket linkern känner igen.
#[no_mangle]
pub extern "C" fn rust_start(info: u64) -> ! {
    
    //lös ut pekaren till multiboot2 informationsstrukturen och översätt till en abstraherad form.
    let multiboot_info = unsafe {
        multiboot_info::MultibootInfo::from_pointer(info as *const MultibootInfoHeader)
    }
    .unwrap();
    
    //hämta ut en abstraktion av den aktiva minneskartläggsningstabell p4.
    let mut active_table = unsafe { PageTableMaster::new() };

    //skapa ny minneskarta, hämta ut ELF-allokerare.
    let mut allocator = memory::remap_everything(multiboot_info, &mut active_table);
    

    unsafe {
        //skapa global minnesallokerare
        populate_global_allocator(&mut active_table, &mut allocator);
        
        //skapa IDT och initiera interrupts.
        interrupt::setup::setup_interrupts();

        //Initiera datorns timer att ticka 
        pitinit(2400);
    }

    // Starta filsystemet och installera alla inbyggda appar
    {
        fs::start();
        builtins::install_all().unwrap();
        fs::install_app::<SplashScreen>().unwrap();
    }

    // Skapa Forths runtime
    let mut forth_machine = ForthMachine::default();

    // definera ordet run
    forth_machine.insert_default_word("run", &run);
    
    // starta bin/startup.for
    forth_machine.add_instructions_to_end(&"\"bin/startup.for\" \"forrunner\" run");
    forth_machine.run_to_end();

    // kör terminalen (operativsystemet är helt startat)
    unsafe {
        let mut string = String::new();
        forth_machine.formatter.enable_cursor().set_position((0, 7));
        loop {
            forth_machine
                .formatter
                .set_default_colors(VgaColorCombo::on_black(VgaColor::White))
                .enable_cursor()
                .write_str(fs::active_directory().as_str())
                .write_str(" > ");
            loop {
                let c: char = KEYBOARD_QUEUE.getch_blocking();
                let count = string.chars().count();
                match c {
                    '\x08' => {
                        forth_machine
                            .formatter
                            .back_up(count)
                            .write_str(&" ".repeat(count))
                            .back_up(count);
                        string.pop();
                    }
                    '\n' => {
                        forth_machine.formatter.next_line();
                        let mut new_string = String::new();
                        core::mem::swap(&mut new_string, &mut string);
                        forth_machine.add_instructions_to_end(&new_string);
                        forth_machine.run_to_end();
                        forth_machine.formatter.next_line();

                        break;
                    },
                    '\0' => {
                        forth_machine.formatter.back_up(count);
                    },
                    _ => {
                        forth_machine.formatter.back_up(count);
                        string.push(c);
                    }
                }
                forth_machine.formatter.write_str(&string);
            }
        }
    };
}

fn run(machine: &mut ForthMachine) {
    let mut app = match get_app(machine) {
        Ok(app) => app,
        Err(err) => {
            machine.formatter.write_str(&format!("Run: {err:?} "));
            return;
        }
    };
    match app.run(machine) {
        Ok(()) => (),
        Err(err) => {
            machine
                .formatter
                .write_str(&format!("App: {err:?} "));
        }
    }
}
fn get_app(machine: &mut ForthMachine) -> Result<Box<dyn LittleManApp>, FileSystemError> {
    let path = machine
        .stack
        .try_pop::<String>()
        .ok_or(FileSystemError::EmptyPath)?;
    let finalized_path = PathString::from(path).add_extension("run");
    let file = fs::get_file_relative(&finalized_path)
        .or(fs::get_file(PathString::from("bin").append(&finalized_path)))?;
    let app = file.launch_app()?;
    Ok(app)
}

unsafe fn tmp_write(s: String) {
    for char in s.chars() {
        while (x86_64::instructions::port::PortReadOnly::<u8>::new(0x3F8 + 5).read() & 0x20) == 0 {}
        PortWriteOnly::new(0x3f8).write(char as u8);
    }
}

fn disable_cursor() {
    unsafe {
        PortWriteOnly::new(0x03D4_u16).write(0x0A_u8);
        PortWriteOnly::new(0x03D5_u16).write(0x20_u8);
    }
}

#[no_mangle]
pub extern "C" fn keyboard_handler() {
    panic!();
}
