use std::fs::{File, OpenOptions};
use std::ptr::slice_from_raw_parts;
use std::mem::size_of;

#[repr(C, packed)]
#[derive(Debug)]
pub struct Meta {
    pub magic: u32,
    pub version: u32,
    pub page_size: u32,
}

pub struct BranchStoredINode {
    pub pos: u32,
    pub ksize: u32,
    pub page_id: u32,
}

pub const PAGE_LEAF: u16 = 0x01;
pub const PAGE_BRANCH: u16 = 0x02;
pub const PAGE_META: u16 = 0x04;
pub const PAGE_FREELIST: u16 = 0x10;

#[repr(C, packed)]
#[derive(Debug)]
pub struct Page {
    pub id: u64,
    pub flags: u16,
    pub inode_count: u32,
    pub page_overflow_count: u32,
}

impl Page {
    pub fn meta(&self) -> Option<&Meta> {
        self._view::<Meta>()
    }

    fn _view<T>(&self) -> Option<&T> where T: Sized {
        let raw_h: *const u8 = (self as *const Page) as *const u8;
        let buf = unsafe {
            slice_from_raw_parts(raw_h, size_of::<Page>() + size_of::<T>() as usize).as_ref().unwrap()
        };

        let (_, body, _) = unsafe { buf[size_of::<Page>()..size_of::<Page>() + size_of::<T>()].align_to::<T>() };

        if body.len() == 1 {
            return Some(&body[0])
        }

        None
    }
}
