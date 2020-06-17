use std::fmt;
use std::fs::{File, OpenOptions};
use std::iter::FromIterator;
use std::mem::size_of;
use std::path::Display;
use std::ptr::slice_from_raw_parts;
use std::str;

const MAX_KEY_SIZE: usize = 32;

pub type PageId = u64;
pub type Key = [u8; MAX_KEY_SIZE];

pub fn key_to_str(val: &[u8]) -> String {
    String::from_iter(
        val.iter()
            .filter(|&&x| x != 0)
            .map(|&x| x as char)
    )
}

pub fn str_to_key(val: &str) -> Key {
    let mut ret: Key = [0; MAX_KEY_SIZE];
    for (i, &b) in val.as_bytes().iter().rev().enumerate() {
        ret[MAX_KEY_SIZE - i - 1] = b;
    }

    ret
}


pub fn val_to_str(val: &[u8]) -> &str {
    return str::from_utf8(val).unwrap();
}


#[repr(C, packed)]
#[derive(Debug)]
pub struct Meta {
    pub magic: u32,
    pub version: u32,
    pub page_size: u32,
    pub root_page: u32,
}

#[repr(C, packed)]
#[derive(Debug)]
pub struct BranchINodeHeader {
    pub pos: u32,
    pub ksize: u32,
    pub page_id: u32,
}

pub struct BranchAccess<'a> {
    pub inode: &'a BranchINodeHeader,
    pub key: &'a [u8],
    pub page_id: u32,
}

impl<'a> BranchAccess<'a> {
    pub fn new(branch: &'a BranchINodeHeader, key: &'a [u8]) -> BranchAccess<'a> {
        BranchAccess {
            inode: branch,
            key,
            page_id: branch.page_id,
        }
    }
}

#[repr(C, packed)]
#[derive(Debug)]
pub struct LeafInodeHeader {
    pub pos: u32,
    pub ksize: u32,
    pub vsize: u32,
    pub page_id: u32,
}

pub struct LeafAccess<'a> {
    pub inode: &'a LeafInodeHeader,
    pub key: &'a [u8],
    pub value: &'a [u8],
}

impl<'a> LeafAccess<'a> {
    pub fn new(leaf: &'a LeafInodeHeader, key: &'a [u8], value: &'a [u8]) -> LeafAccess<'a> {
        LeafAccess {
            inode: leaf,
            key,
            value,
        }
    }
}

pub const PAGE_LEAF: u16 = 0x01;
pub const PAGE_BRANCH: u16 = 0x02;
pub const PAGE_META: u16 = 0x04;
pub const PAGE_FREELIST: u16 = 0x10;

// Page либо из mmap, либо из Vec<u8>; Это абстракция над несколькими видами памяти.

#[repr(C, packed)]
#[derive(Debug)]
pub struct PageHeader {
    pub id: u64,
    pub flags: u16,
    pub inode_count: u32,
    pub page_overflow_count: u32,
}

pub struct PageAccess {
    pub header: &'static PageHeader,
    buffer: &'static [u8],
}

impl PageAccess {
    pub fn from_memory(buffer: &'static [u8]) -> PageAccess {
        let (_, body, _) = unsafe { buffer[0..size_of::<PageHeader>()].align_to::<PageHeader>() };

        PageAccess {
            header: &body[0],
            buffer,
        }
    }

    pub fn meta(&self) -> Option<&Meta> {
        self._view::<Meta>()
    }

    pub fn type_name(&self) -> &str {
        if self.header.flags & PAGE_BRANCH != 0 {
            return "branch";
        }

        if self.header.flags & PAGE_LEAF != 0 {
            return "leaf";
        }

        if self.header.flags & PAGE_FREELIST != 0 {
            return "freelist";
        }

        if self.header.flags & PAGE_META != 0 {
            return "meta";
        }

        "unknown"
    }

    pub fn is_leaf(&self) -> bool {
        self.header.flags & PAGE_LEAF != 0
    }

    fn _view<T>(&self) -> Option<&T> where T: Sized {
        let raw_h: *const u8 = (self.header as *const PageHeader) as *const u8;
        let buf = unsafe {
            slice_from_raw_parts(raw_h, size_of::<PageHeader>() + size_of::<T>() as usize).as_ref().unwrap()
        };

        let (_, body, _) = unsafe { buf[size_of::<PageHeader>()..size_of::<PageHeader>() + size_of::<T>()].align_to::<T>() };

        if body.len() == 1 {
            return Some(&body[0]);
        }

        None
    }

    pub fn leaf_elements(&self) -> Vec<LeafAccess> {
        let inode = (&self.buffer[size_of::<PageHeader>()..] as *const [u8]) as *const LeafInodeHeader;

        let inodes = unsafe {
            slice_from_raw_parts(inode, self.header.inode_count as usize).as_ref().unwrap()
        };

        let mut ret = Vec::<LeafAccess>::with_capacity(self.header.inode_count as usize);

        for leaf in inodes.iter() {
            let k = &self.buffer[(leaf.pos as usize)..(leaf.pos as usize) + (leaf.ksize as usize)];
            let v = &self.buffer[((leaf.pos + leaf.ksize) as usize)..((leaf.pos + leaf.ksize) as usize) + (leaf.vsize as usize)];

            ret.push(LeafAccess::new(leaf, k, v));
        }

        ret
    }

    pub fn branch_elements(&self) -> Vec<BranchAccess> {
        let inode = (&self.buffer[size_of::<PageHeader>()..] as *const [u8]) as *const BranchINodeHeader;

        let inodes = unsafe {
            slice_from_raw_parts(inode, self.header.inode_count as usize).as_ref().unwrap()
        };

        let mut ret = Vec::<BranchAccess>::with_capacity(self.header.inode_count as usize);

        for branch in inodes.iter() {
            let k = &self.buffer[(branch.pos as usize)..(branch.pos as usize) + (branch.ksize as usize)];

            ret.push(BranchAccess::new(branch, k));
        }

        ret
    }
}


impl fmt::Display for PageAccess {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} page: {:?}", self.type_name(), self.header)?;

        fmt::Result::Ok(())
    }
}
