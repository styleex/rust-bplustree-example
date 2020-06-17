use std::fmt;
use std::iter::FromIterator;
use std::mem::size_of;
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

impl BranchINodeHeader {
    pub fn key(&self) -> &[u8] {
        let buf = unsafe {
            let tmp = (self as *const BranchINodeHeader) as *const u8;
            slice_from_raw_parts(tmp, std::usize::MAX).as_ref().unwrap()
        };

        return &buf[self.pos as usize..(self.pos + self.ksize) as usize];
    }
}


#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct LeafInodeHeader {
    pub pos: u32,
    pub ksize: u32,
    pub vsize: u32,
    pub page_id: u32,
}

impl LeafInodeHeader {
    pub fn key(&self) -> &[u8] {
        let buf = unsafe {
            let tmp = (self as *const LeafInodeHeader) as *const u8;
            slice_from_raw_parts(tmp, std::usize::MAX).as_ref().unwrap()
        };

        return &buf[self.pos as usize..(self.pos + self.ksize) as usize];
    }

    pub fn value(&self) -> &[u8] {
        let buf = unsafe {
            let tmp = (self as *const LeafInodeHeader) as *const u8;
            slice_from_raw_parts(tmp, std::usize::MAX).as_ref().unwrap()
        };

        return &buf[(self.pos + self.ksize) as usize..(self.pos + self.ksize + self.vsize) as usize];
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

impl PageHeader {
    pub fn meta(&self) -> Option<&Meta> {
        self._view::<Meta>()
    }

    pub fn type_name(&self) -> &str {
        if self.flags & PAGE_BRANCH != 0 {
            return "branch";
        }

        if self.flags & PAGE_LEAF != 0 {
            return "leaf";
        }

        if self.flags & PAGE_FREELIST != 0 {
            return "freelist";
        }

        if self.flags & PAGE_META != 0 {
            return "meta";
        }

        "unknown"
    }

    pub fn is_leaf(&self) -> bool {
        self.flags & PAGE_LEAF != 0
    }

    pub fn is_branch(&self) -> bool {
        self.flags & PAGE_BRANCH != 0
    }

    fn _view<T>(&self) -> Option<&T> where T: Sized {
        let buf = unsafe {
            let tmp = (self as *const PageHeader) as *const u8;
            slice_from_raw_parts(tmp, size_of::<PageHeader>() + size_of::<T>() as usize).as_ref().unwrap()
        };

        let (_, body, _) = unsafe { buf[size_of::<PageHeader>()..size_of::<PageHeader>() + size_of::<T>()].align_to::<T>() };

        if body.len() == 1 {
            return Some(&body[0]);
        }

        None
    }

    pub fn leaf_inodes(&self) -> &[LeafInodeHeader] {
        if !self.is_leaf() {
            panic!("Access as leaf on non leaf element");
        }
        let buf = unsafe {
            let tmp = (self as *const PageHeader) as *const u8;
            slice_from_raw_parts(tmp, 4096).as_ref().unwrap()
        };

        let inode = (&buf[size_of::<PageHeader>()..] as *const [u8]) as *const LeafInodeHeader;

        let inodes = unsafe {
            slice_from_raw_parts(inode, self.inode_count as usize).as_ref().unwrap()
        };

        return inodes;
    }

    pub fn branch_inodes(&self) -> &[BranchINodeHeader] {
        if !self.is_branch() {
            panic!("Access as branch on non branch element");
        }

        let buf = unsafe {
            let tmp = (self as *const PageHeader) as *const u8;
            slice_from_raw_parts(tmp, std::usize::MAX).as_ref().unwrap()
        };

        let inode = (&buf[size_of::<PageHeader>()..] as *const [u8]) as *const BranchINodeHeader;

        let inodes = unsafe {
            slice_from_raw_parts(inode, self.inode_count as usize).as_ref().unwrap()
        };

        return inodes;
    }
}


impl fmt::Display for PageHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} page: {:?}", self.type_name(), self)?;

        fmt::Result::Ok(())
    }
}
