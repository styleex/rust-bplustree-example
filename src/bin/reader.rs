use std::fs::{File, OpenOptions};
use std::iter::FromIterator;
use std::mem::size_of;

use memmap::Mmap;

use types::{Key, key_to_str, Meta, PageAccess, PageHeader, PageId, str_to_key, val_to_str};

mod types;

pub struct DB {
    f: File,
    mmap_data: Mmap,
    page_size: usize,
}

impl DB {
    pub fn open(path: &str) -> DB {
        let f = OpenOptions::new().read(true).open(path).unwrap();

        let mut db = DB {
            f: f.try_clone().unwrap(),
            mmap_data: unsafe {
                memmap::MmapOptions::new().len(f.try_clone().unwrap().metadata().unwrap().len() as usize).
                    offset(0).map(&f).unwrap()
            },
            page_size: 0,
        };

        db.page_size = db.page(0).meta().unwrap().page_size as usize;

        return db;
    }

    fn page(&self, id: PageId) -> PageAccess {
        PageAccess::from_memory(&self.mmap_data[(id as usize) * self.page_size..])
    }

    // Ищет листовой элемент, в котором должен (но не обязан, если его вообще не добавляли)
    // располагаться нужный ключ
    fn _tree_search_page(&self, k: Key, page_id: PageId) -> PageId {
        let page = self.page(page_id);
        let branch = page.branch_elements();
        for b in branch.iter().as_ref() {
            println!("{}", key_to_str(b.key));
        }

        let pos = branch.iter().position(|x| x.key >= &k).unwrap_or((page.header.inode_count - 1) as usize);
        return branch[pos].page_id as PageId;
    }

    pub fn search(&self, k: Key) -> PageId {
        let mut page_id = self.page(0).meta().unwrap().root_page as PageId;

        loop {
            println!("{}", self.page(page_id));
            if self.page(page_id).is_leaf() {
                return page_id;
            }

            page_id = self._tree_search_page(k, page_id);
        }
    }
}


fn main() {
    let db = DB::open("/home/vladimirov/workspace/rust_apps/db.rust");

    let k = str_to_key("92");
    let page_id = db.search(k);

    let leafs = db.page(page_id).leaf_elements();
    for i in leafs.iter() {
        println!("{}", key_to_str(i.key));
    }

    let pg = db.page(page_id).leaf_elements();
    let ret = leafs.iter()
        .find(|leaf| leaf.key == &k);

    if ret.is_some() {
        println!("ret: {}", val_to_str(ret.unwrap().value));
    } else {
        println!("ret: not found");
    }
}
