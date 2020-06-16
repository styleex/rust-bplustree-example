use std::fs::{File, OpenOptions};
use std::mem::size_of;

mod types;

fn mmap_view<T>(mmap: &[u8], offset: usize) -> &T where T: Sized + std::fmt::Debug {
    let (_, body, _) = unsafe { mmap[offset..offset + size_of::<T>()].align_to::<T>() };

    &body[0]
}

fn main() {
    let path = "/home/anton/workspace/rust-bplustree-example/db.rust";
    let mut f = OpenOptions::new().read(true).open(path).unwrap();

    let mmap_data = unsafe {
        memmap::MmapOptions::new().len(f.metadata().unwrap().len() as usize).
            offset(0).map(&f).unwrap()
    };

    let page: &types::Page = mmap_view(&mmap_data, 0);
    println!("{} {:?} {:?}", page.type_name(), page, page.meta());

    let meta = page.meta().unwrap();

    let base_offset = 1 * meta.page_size as usize;
    let page: &types::Page = mmap_view(&mmap_data, base_offset);
    println!("{}: {:?}", page.type_name(), page);

    // for i in 0..page.inode_count {
    //     let branch_data: &types::LeafStoredINode = mmap_view(&mmap_data, base_offset + offset);
    //     offset += size_of::<types::LeafStoredINode>() as usize;
    //
    //     println!("{:?}", branch_data);
    //
    //     let key: &[u8] = &mmap_data[base_offset+(branch_data.pos) as usize..(base_offset as u32 +branch_data.pos+branch_data.ksize) as usize];
    //     println!("{}", types::key_to_str(key));
    //
    //     let val: &[u8] = &mmap_data[base_offset+(branch_data.pos + branch_data.ksize) as usize..(base_offset as u32 +branch_data.pos+branch_data.ksize+branch_data.vsize) as usize];
    //     println!("{}", types::val_to_str(val));
    // }

    let page: &types::Page = mmap_view(&mmap_data, (meta.root_page * meta.page_size) as usize);
    println!("{}: {:?}", page.type_name(), page);

    let root = page.branch_elements(&mmap_data[(meta.root_page * meta.page_size) as usize..]);
    for br in root.iter() {
        println!("{:?}: key={}", br.inode, types::key_to_str(br.key));
    }

    let page: &types::Page = mmap_view(&mmap_data, (1 * meta.page_size) as usize);
    println!("{}: {:?}", page.type_name(), page);

    let leafs = page.leaf_elements(&mmap_data[base_offset..]);
    for leaf in leafs.iter() {
        println!("{:?}: key={}, value={}", leaf.inode, types::key_to_str(leaf.key), types::val_to_str(leaf.value));
    }
}
