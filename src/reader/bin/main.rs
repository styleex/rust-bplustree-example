use std::fs::{File, OpenOptions};
use std::mem::size_of;


#[repr(C, packed)]
#[derive(Debug)]
struct Header {
    magic: u32,
    version: u32,
    page_size: u32,
}

const MAX_KEY_SIZE: usize = 32;

type Key = [u8; MAX_KEY_SIZE];

type NodeId = usize;

// Для листа содержит и ключ и значение. Для родителя только ключи
#[repr(C, packed)]
#[derive(Debug)]
struct INode {
    key: Key,
    value: Option<Vec<u8>>,
}

// https://gist.github.com/savarin/69acd246302567395f65ad6b97ee503d
#[repr(C, packed)]
#[derive(Debug)]
struct Node {
    id: NodeId,
    is_leaf: bool,
    parent_id: Option<NodeId>,
    // childs: Vec<NodeId>,

    // inodes: Vec<INode>,
}


fn mmap_view<T>(mmap: &[u8], offset: usize) -> &T where T: Sized + std::fmt::Debug {
    let (_, body, _) = unsafe { mmap[offset..offset+size_of::<T>()].align_to::<T>()};

    &body[0]
}

fn main() {
    let path = "/home/anton/workspace/rust-bplustree-example/db.rust";
    let mut f = OpenOptions::new().read(true).open(path).unwrap();

    let mmap_data= unsafe {
        memmap::MmapOptions::new().len(f.metadata().unwrap().len() as usize).
        offset(0).map(&f).unwrap()
    };

    let hdr: &Header = mmap_view(&mmap_data, 0);
    println!("{:?}", hdr);

    let node: &Node = mmap_view(&mmap_data, size_of::<Header>());
    println!("{:?}", node);
}
