use core::mem::size_of;

use crate::{
    display::{KernelDebug, KernelFormatter},
    memory::MemoryAreaIter,
};

use super::{transmute, type_after, TagHeader, TagType};

#[allow(dead_code)]
pub struct MemoryMapTag {
    header: &'static MemoryMapHeader,
    entries: &'static [MemoryMapEntry],
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub struct BasicMemoryTag {
    tag_type: TagType,
    size: u32,
    lower: u32,
    higher: u32,
}
impl BasicMemoryTag {
    pub unsafe fn from_ref(pointer: &TagHeader) -> Self {
        *transmute(pointer)
    }
}
impl<'a> KernelDebug<'a> for BasicMemoryTag {
    fn debug(&self, formatter: KernelFormatter<'a>) -> KernelFormatter<'a> {
        formatter
            .debug_struct("BasicMemTag")
            .debug_field("higher", &self.higher)
            .debug_field("lower", &self.lower)
            .finish()
    }
}

#[repr(C)]
pub struct MemoryMapHeader {
    tag_type: TagType,
    size: u32,
    entry_size: u32,
    entry_version: u32,
}

#[repr(C)]
pub struct MemoryMapEntry {
    pub base_address: u64,
    pub length: u64,
    pub mem_type: MemoryType,
    _reserved: u32,
}

#[allow(dead_code)]
#[repr(u32)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum MemoryType {
    Available = 1,
    Unknown = 2,
    ACPI = 3,
    Reserved = 4,
    Defective = 5,
}

impl MemoryMapTag {
    pub unsafe fn from_ref(head: &TagHeader) -> Self {
        let pointer: *const MemoryMapHeader = transmute(head as *const TagHeader);
        let header = &*pointer;
        let entries_start: *const MemoryMapEntry = type_after(pointer);
        let slice_len =
            (header.size as usize - size_of::<MemoryMapHeader>()) / size_of::<MemoryMapEntry>();
        let entries = core::slice::from_raw_parts(entries_start, slice_len);
        Self { header, entries }
    }
    pub fn area_iter(&self) -> MemoryAreaIter {
        MemoryAreaIter::new(self.entries)
    }
}
impl<'a> KernelDebug<'a> for MemoryMapTag {
    fn debug(&self, formatter: KernelFormatter<'a>) -> KernelFormatter<'a> {
        formatter
            .debug_struct("MemoryMapTag")
            .debug_field("entries", &self.entries)
            .finish()
    }
}
impl<'a> KernelDebug<'a> for MemoryMapEntry {
    fn debug(&self, formatter: KernelFormatter<'a>) -> KernelFormatter<'a> {
        formatter
            .debug_struct("MemoryMapEntry")
            .debug_field("base", &self.base_address)
            .debug_field("len", &self.length)
            .debug_field("type", &self.mem_type)
            .finish()
    }
}
impl<'a> KernelDebug<'a> for MemoryType {
    fn debug(&self, formatter: KernelFormatter<'a>) -> KernelFormatter<'a> {
        let str = match self {
            MemoryType::Available => "Available",
            MemoryType::Unknown => "Unknown",
            MemoryType::ACPI => "ACPI",
            MemoryType::Reserved => "Reserved",
            MemoryType::Defective => "Defect",
        };
        formatter.debug_str(str)
    }
}
