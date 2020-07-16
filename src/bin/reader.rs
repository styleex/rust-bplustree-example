//#[macro_use] extern crate log;
use std::fs::{File, OpenOptions};
use memmap::Mmap;
use log::{info, trace, warn};
use types::{Key, key_to_str, PageHeader, PageId, str_to_key, val_to_str};

mod types;
mod node;

use node::{INode, HeapValue};

pub struct DB {
    f: File,
    mmap_data: Mmap,
    page_size: usize,
}

impl<'a> DB {
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
        unsafe {
            let raw_bytes = (&self.mmap_data[(id as usize) * self.page_size..][0]) as *const u8;
            let raw_page_header = std::mem::transmute::<*const u8, *const PageHeader>(raw_bytes);

            std::mem::transmute::<*const PageHeader, &PageHeader>(raw_page_header)
        }
    }

    // Ищет листовой элемент, в котором должен (но не обязан, если его вообще не добавляли)
    // располагаться нужный ключ
    fn _tree_search_page(&self, k: Key, page_id: PageId) -> PageId {
        let page = self.page(page_id);

        let mut ret_idx = (page.inode_count - 1) as usize;
        for (idx, inode) in page.branch_inodes().iter().enumerate() {
            trace!("page_id={} key={}", inode.page_id, key_to_str(inode.key()));

            if inode.key() > &k {
                trace!("Desired key found. Current page processing stopped");
                ret_idx = (idx - 1) as usize;
                break;
            }
        }

        return page.branch_inodes()[ret_idx].page_id as PageId;
    }

    pub fn search(&self, k: Key) -> PageId {
        let mut page_id = self.page(0).meta().unwrap().root_page as PageId;

        loop {
            trace!("Search on page: {:?}", self.page(page_id));
            if self.page(page_id).is_leaf() {
                return page_id;
            }

            page_id = self._tree_search_page(k, page_id);
        }
    }

    pub fn get(&self, k: Key) -> Option<&[u8]> {
        trace!("Search \"{}\"", key_to_str(&k));
        let page_id = self.search(k);

        return self.page(page_id).leaf_inodes()
            .iter()
            .find(|inode| inode.key() == &k)
            .map(|x| x.value());
    }

    pub fn update(&'a mut self, f: fn(&mut Tx)) {
        let mut tx = Tx::new(self);
        f(&mut tx);
        tx.commit();
    }

    pub fn close(&self) {
        println!("close");
    }
}

pub struct Tx<'a> {
    db: &'a DB,
    node_cache: node::NodeCache<'a>
}

// 1. При чтении - читаются данные из страницы. Страница при этом не должна удаляться
// 2. При записи:
//      1. Запись осуществляется всегда в листовой узел, который занимает не менее одной страницы;
//      2. Перезаписывается всегда весь лист (все его страницы) целиком;
//      3. Поэтому при обновлении листа мы:
//           - Создаем новую ноду; Новая нода должна содержать ссылки на старые данные из mmap
//              (чтоб потом скопировать) и ссылки на новые данные (из heap). Заранее выделить страницы
//              под лист мы не можем, т.к. не знаем сколько он впоследствии будет занимать места;
//
// Где хранить новые key и value, кто их owner?
//    - В Tx, а ссылки на эти данные в INode;
//    - Только в inode;
//
// Node и INode - промежуточные структуры данных, которые связывают:
//  - Runtime данные (изменение элементов дерева); Pipeline: node -> page -> file
//  - Старые данные (ссылки на данные из mmap), чтобы избежать лишних копирований данных
//    Вместо (mmap -> node -> page -> file) у нас (mmap -> (-> &node (link to mmap)->) -> page -> file)
impl<'a> Tx<'a> {
    pub fn new(db: &mut DB) -> Tx {
        Tx {
            db,
            node_cache: node::NodeCache::new(),
        }
    }

    pub fn put(&mut self, key: Key, val: Vec<u8>) {
        let page_id = self.db.search(key);

        let node_id = {
            let pg = self.db.page(page_id);
            self.node_cache.read_node(pg)
        };

        let pos = self.node_cache.nodes[node_id].inodes.binary_search_by_key(&key.as_ref(), |x| x.key());
        match pos {
            Ok(pos) => {
                self.node_cache.nodes[node_id].inodes[pos].value = HeapValue::Heap(val);
            },
            Err(pos) => {
                self.node_cache.nodes[node_id].inodes.insert(pos, INode {
                    key: HeapValue::Heap(Vec::from(key)),
                    value: HeapValue::Heap(val),
                    page_id: None,
                })
            }
        }
    }

    pub fn commit(&mut self) {
        println!("commit");
    }
}


fn main() {
    env_logger::init();
    let mut db = DB::open(std::env::current_dir().unwrap().as_path().join("db.rust").as_path().to_str().unwrap());

    let k = str_to_key("3");
    let ret = db.get(k);

    if ret.is_some() {
        println!("ret: {}", val_to_str(ret.unwrap()));
    } else {
        println!("ret: not found");
    }

    db.update(|tx| {
        tx.put(str_to_key("3"), "asd65".bytes().collect());
    });

    db.close();
}
