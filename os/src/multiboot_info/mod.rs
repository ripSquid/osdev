use core::{mem::size_of, str::from_utf8};

use crate::display::{
    macros::{debug, print_hex, print_str},
    KernelDebug,
};

pub struct MultibootInfo<'a> {
    pub header: MultibootInfoHeader,
    pub tags: MultiBootTags<'a>,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct MultibootInfoHeader {
    total_size: u32,
    _reserved: u32,
}
impl<'a> MultibootInfo<'a> {
    pub unsafe fn from_pointer(pointer: *const MultibootInfoHeader) -> Option<Self> {
        if pointer.align_offset(8) != 0 || pointer.is_null() {
            return None;
        }
        let header = *pointer;
        let raw = pointer as u64;
        let tags =
            core::slice::from_raw_parts((raw + 24) as *const u8, (header.total_size - 24) as usize);
        let tags = MultiBootTags::from_slice(tags)?;
        print_hex!(header.total_size);
        Some(MultibootInfo { header, tags })
    }
    pub fn size(&self) -> usize {
        self.header.total_size as usize
    }
}
impl<'a> KernelDebug<'a> for MemoryMapTag<'a> {
    fn debug(&self, formatter: crate::display::KernelFormatter<'a>) -> crate::display::KernelFormatter<'a> {
        formatter
            .debug_struct("MemoryMapTag")
            .debug_field("entries", &self.entries)
            .finish()
    }
}
impl<'a> KernelDebug<'a> for MemoryMapEntry {
    fn debug(&self, formatter: crate::display::KernelFormatter<'a>) -> crate::display::KernelFormatter<'a> {
        formatter
            .debug_struct("MemoryMapEntry")
            .debug_field("base", &self.base_address)
            .debug_field("len", &self.length)
            .debug_field("type", &self.mem_type)
            .finish()
    }
}
pub struct MultiBootTags<'a>(&'a [u8]);

impl<'a> MultiBootTags<'a> {
    pub fn bytes(&self) -> &[u8] {
        self.0
    }
    pub fn from_slice(slice: &'a [u8]) -> Option<Self> {
        Some(Self(slice))
    }
    pub fn memory_tag(&self) -> Option<&'a [MemoryMapEntry]> {
        let mut searching = true;
        let mut counter = 0;

        while searching {
            let tag_head: &TagHeader = unsafe { &*transmute(&self.0[counter] as *const u8) };
            match tag_head.tag_type {
                TagType::BootCommandLine => print_str!("command line"),
                TagType::BootoaderName => {
                    let tag = unsafe { BootloaderNameTab::from_ref(&tag_head) };
                    print_str!(tag.name);
                    print_str!("name");
                }
                TagType::Module => print_str!("modul"),
                TagType::BasicMemoryTag => print_str!("basic mem"),
                TagType::BiosBootDevice => print_str!("boot device"),
                TagType::MemoryMap => {
                    let info = unsafe { MemoryMapTag::from_ref(&tag_head) };
                    debug!(&info);
                    return None;
                    print_str!("memory!!!");
                }
                TagType::VbeInfo => print_str!("vbe"),
                TagType::FramebufferInfo => print_str!("frame info"),
                TagType::ElfSymbol => print_str!("elf symbol"),
                TagType::ApmTable => print_str!("apm"),
                TagType::End => {
                    print_str!("end tag");
                    if tag_head.size == 8 {
                        return None;
                    } else {
                        print_str!("SOMETHIGN IS WROGN WITH END TAG!");
                        panic!();
                    }
                }
                _ => print_str!("PISS AND SHIT AND FUCK"),
            }

            //rounds upward to nearest multiple of 8
            let moving = ((tag_head.size + 7) & MASK8) as usize;
            counter += moving;
        }

        None
    }
}

//provides a mask that removes the last 3 bits of any u32 (rounding it to nearest multiple of 8)
const MASK8: u32 = u32::MAX - 0x07;

#[repr(u32)]
#[derive(Clone, Copy)]
enum TagType {
    End = 0,
    BootCommandLine = 1,
    BootoaderName = 2,
    Module = 3,
    BasicMemoryTag = 4,
    BiosBootDevice = 5,
    MemoryMap = 6,
    VbeInfo = 7,
    FramebufferInfo = 8,
    ElfSymbol = 9,
    ApmTable = 10,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TagHeader {
    tag_type: TagType,
    size: u32,
}

#[repr(C)]
pub struct MemoryMapEntry {
    pub base_address: u64,
    pub length: u64,
    pub mem_type: u32,
    _reserved: u32,
}

#[repr(C)]
pub struct MemoryMapHeader {
    tag_type: TagType,
    size: u32,
    entry_size: u32,
    entry_version: u32,
}

pub struct MemoryMapTag<'a> {
    header: &'a MemoryMapHeader,
    entries: &'a [MemoryMapEntry],
}

impl<'a> MemoryMapTag<'a> {
    pub unsafe fn from_ref(head: &'a TagHeader) -> Self {
        let pointer: *const MemoryMapHeader = transmute(head as *const TagHeader);
        let header = &*pointer;
        let entries_start: *const MemoryMapEntry = type_after(pointer);
        let slice_len =
            (header.size as usize - size_of::<MemoryMapHeader>()) / size_of::<MemoryMapEntry>();
        let entries = core::slice::from_raw_parts(entries_start, slice_len);
        Self { header, entries }
    }
}

pub struct BootloaderNameTab<'a> {
    head: &'a TagHeader,
    name: &'a str,
}
impl<'a> BootloaderNameTab<'a> {
    pub unsafe fn from_ref(head: &'a TagHeader) -> Self {
        let pointer: *const u8 = type_after(head as *const TagHeader);
        let sting_len = head.size as usize - size_of::<TagHeader>() - 1;
        let string_bytes = core::slice::from_raw_parts(pointer as *const u8, sting_len);
        let name = from_utf8(string_bytes).unwrap();
        Self { head, name }
    }
}

/// Gives a pointer to a data type laid out after the one pointed to in `pointer`
pub unsafe fn type_after<B, A>(pointer: *const B) -> *const A {
    pointer.offset(1) as u64 as *const A
}

/// turn a pointer of one type into another, mega hacky!!!
pub unsafe fn transmute<B, A>(pointer: *const B) -> *const A {
    pointer as u64 as *const A
}
