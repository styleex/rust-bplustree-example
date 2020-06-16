use std::fs::{File, OpenOptions};
use std::mem::size_of;
mod types;

fn mmap_view<T>(mmap: &[u8], offset: usize) -> &T where T: Sized + std::fmt::Debug {
    let (_, body, _) = unsafe { mmap[offset..offset+size_of::<T>()].align_to::<T>()};

    &body[0]
}

fn main() {
    let path = "/home/vladimirov/workspace/rust_apps/db.rust";
    let mut f = OpenOptions::new().read(true).open(path).unwrap();

    let mmap_data= unsafe {
        memmap::MmapOptions::new().len(f.metadata().unwrap().len() as usize).
        offset(0).map(&f).unwrap()
    };

    let page: &types::Page = mmap_view(&mmap_data, 0);
    println!("{} {:?} {:?}", page.type_name(), page, page.meta());

    let meta = page.meta().unwrap();

    let base_offset = 1 * meta.page_size as usize;
    let page: &types::Page = mmap_view(&mmap_data, base_offset);
    println!("{}: {:?}", page.type_name(), page);

    let mut offset: usize = size_of::<types::Page>();
    for i in 0..page.inode_count {
        let branch_data: &types::LeafStoredINode = mmap_view(&mmap_data, base_offset + offset);
        offset += size_of::<types::LeafStoredINode>() as usize;

        println!("{:?}", branch_data);

        let key: &[u8] = &mmap_data[base_offset+(branch_data.pos) as usize..(base_offset as u32 +branch_data.pos+branch_data.ksize) as usize];
        println!("{}", types::key_to_str(key));

        let val: &[u8] = &mmap_data[base_offset+(branch_data.pos + branch_data.ksize) as usize..(base_offset as u32 +branch_data.pos+branch_data.ksize+branch_data.vsize) as usize];
        println!("{}", types::val_to_str(val));
    }
//    let hdr: &Meta = mmap_view(&mmap_data, 0);
//    println!("{:?}", hdr);
//
//    let node: &Node = mmap_view(&mmap_data, size_of::<Meta>());
//    println!("{:?}", node);
}
