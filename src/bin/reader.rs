use std::fs::{File, OpenOptions};
use memmap::Mmap;
use types::{Key, key_to_str, PageHeader, PageId, str_to_key, val_to_str};

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

    fn page(&self, id: PageId) -> &PageHeader {
        let (_, body, _) = unsafe { self.mmap_data[(id as usize) * self.page_size..].align_to::<PageHeader>() };
        return &body[0];
    }

    // Ищет листовой элемент, в котором должен (но не обязан, если его вообще не добавляли)
    // располагаться нужный ключ
    fn _tree_search_page(&self, k: Key, page_id: PageId) -> PageId {
        let page = self.page(page_id);

        let mut ret_idx = (page.inode_count - 1) as usize;
        for (idx, inode) in page.branch_inodes().iter().enumerate() {
            println!("{} page_id={}", key_to_str(inode.key()), inode.page_id);

            if inode.key() > &k {
                ret_idx = (idx - 1) as usize;
                break
            }
        }

        return page.branch_inodes()[ret_idx].page_id as PageId;
    }

    pub fn search(&self, k: Key) -> PageId {
        let mut page_id = self.page(0).meta().unwrap().root_page as PageId;

        loop {
            println!("{:?}", self.page(page_id));
            if self.page(page_id).is_leaf() {
                return page_id;
            }

            page_id = self._tree_search_page(k, page_id);
        }
    }

    pub fn get(&self, k: Key) -> Option<&[u8]> {
        let page_id = self.search(k);

        return self.page(page_id).leaf_inodes()
            .iter()
            .find(|inode| inode.key() == &k)
            .map(|x| x.value());
    }
}


fn main() {
    let db = DB::open("/home/anton/workspace/rust-bplustree-example/db.rust");

    let k = str_to_key("100");
    let ret = db.get(k);

    if ret.is_some() {
        println!("ret: {}", val_to_str(ret.unwrap()));
    } else {
        println!("ret: not found");
    }
}
