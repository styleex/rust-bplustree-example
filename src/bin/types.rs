use std::fs::{File, OpenOptions};
use std::ptr::slice_from_raw_parts;
use std::mem::size_of;
use std::str;
use std::iter::FromIterator;


const MAX_KEY_SIZE: usize = 32;

pub type Key = [u8; MAX_KEY_SIZE];
pub fn key_to_str(val: &[u8]) -> String {
    String::from_iter(
        val.iter()
            .filter(|&&x| x != 0)
            .map(|&x| x as char)
    )
}


#[repr(C, packed)]
#[derive(Debug)]
pub struct Meta {
    pub magic: u32,
    pub version: u32,
    pub page_size: u32,
}

#[repr(C, packed)]
#[derive(Debug)]
pub struct BranchStoredINode {
    pub pos: u32,
    pub ksize: u32,
    pub page_id: u64,
}

#[repr(C, packed)]
#[derive(Debug)]
pub struct LeafStoredINode {
    pub pos: u32,
    pub ksize: u32,
    pub vsize: u32,
    pub page_id: u32,
}

pub const PAGE_LEAF: u16 = 0x01;
pub const PAGE_BRANCH: u16 = 0x02;
pub const PAGE_META: u16 = 0x04;
pub const PAGE_FREELIST: u16 = 0x10;

// Page либо из mmap, либо из Vec<u8>; Это абстракция над несколькими видами памяти.
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

    pub fn type_name(&self) -> &str {
        if self.flags & PAGE_BRANCH != 0 {
            return "branch"
        }

        if self.flags & PAGE_LEAF != 0 {
            return "leaf"
        }

        if self.flags & PAGE_FREELIST != 0 {
            return "freelist"
        }

        if self.flags & PAGE_META != 0 {
            return "meta"
        }

        "unknown"
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
