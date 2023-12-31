use crate::display::{DefaultVgaBuffer, DefaultVgaWriter, VgaColor, VgaColorCombo};
use core::{fmt::Write, panic::PanicInfo, str::from_utf8_unchecked};

use self::formatter::PanicBuffer;

mod formatter;
// Address of the default 80x25 vga text mode buffer left to us after grub.
pub const VGA_BUFFER_ADDRESS: u64 = 0xB8000;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let mut writer =
        DefaultVgaWriter::new(unsafe { &mut *(VGA_BUFFER_ADDRESS as *mut DefaultVgaBuffer) });
    let error_color = VgaColorCombo::new(VgaColor::White, VgaColor::Blue);
    //writer.clear_screen(VgaColor::Blue);
    writer
        .enable_cursor()
        .disable_cursor()
        .set_default_colors(error_color);
    writer.write_horizontally_centerd("PANIC OCCURED:", 3);
    if let Some(location) = info.location() {
        writer
            .write_horizontally_centerd("file:", 4)
            .write_horizontally_centerd(location.file(), 5)
            .write_horizontally_centerd("line:", 6)
            .write_horizontally_centerd(
                unsafe { from_utf8_unchecked(U32Str::from(location.line()).as_ref()) },
                7,
            );

        writer
            .write_horizontally_centerd("column:", 8)
            .write_horizontally_centerd(
                unsafe { from_utf8_unchecked(U32Str::from(location.column()).as_ref()) },
                9,
            );

        if let Some(message) = info.message() {
            writer
                .write_horizontally_centerd("error:", 10)
                .next_line()
                .set_default_colors(VgaColorCombo::on_black(VgaColor::Red));

            let mut buffer = PanicBuffer::new(&mut writer);
            let _ = write!(&mut buffer, "{message}");
        } else {
            writer.write_horizontally_centerd("(no attached error message)", 10);
        }
    }

    loop {}
}

/// a datatype with enough capacity to hold any u32 value
struct U32Str {
    value: [u8; 10],
    len: usize,
}
impl U32Str {
    pub fn as_bytes(&self) -> &[u8] {
        &self.value[..self.len]
    }
    pub fn from(u: u32) -> Self {
        let mut value: [u8; 10] = [0; 10];
        let mut len = 0;
        let mut accounted = 0;
        for i in (0..10).rev() {
            let mut mult = 1;
            for _ in 0..i {
                mult *= 10;
            }
            let this_char = (u - accounted) / mult;
            if this_char > 0 {
                accounted += this_char * mult;
            } else {
                continue;
            }
            if accounted > 0 {
                value[len] = (this_char as u8) + 0x30;
                len += 1;
            }
        }
        U32Str { value, len }
    }
}
impl AsRef<[u8]> for U32Str {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}
