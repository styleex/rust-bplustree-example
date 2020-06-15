use std::fmt::Display;
use core::fmt;
use std::iter::FromIterator;
use std::str;

mod types;

const MAX_KEY_SIZE: usize = 32;

type Key = [u8; MAX_KEY_SIZE];

type NodeId = usize;

// Для листа содержит и ключ и значение. Для родителя только ключи
#[repr(C, packed)]
struct INode {
    key: Key,
    value: Option<Vec<u8>>,
}

// https://gist.github.com/savarin/69acd246302567395f65ad6b97ee503d
#[repr(C, packed)]
struct Node {
    id: NodeId,
    is_leaf: bool,
    parent_id: Option<NodeId>,
    childs: Vec<NodeId>, // runtime only

    inodes: Vec<INode>,
}

impl Node {
    pub fn size(&self) -> u64 {
        let mut size = size_of::<Page>() as u64;

        let stored_inode_size = if self.is_leaf {
            size_of::<LeafStoredINode>()
        } else {
            size_of::<BranchStoredINode>()
        } as u64;

        for inode in self.inodes.iter() {
            size += stored_inode_size + inode.key.len() as u64;

            if inode.value.is_some() {
                size += inode.value.as_ref().unwrap().len() as u64
            }
        };

        size
    }

    pub fn serialize(&self, page: &mut Page) {
    }
}

struct BPlusTree {
    order: usize, // Сколько потомков может хранить нода

    nodes: Vec<Node>,
    // Список всех нод дерева
    root_id: NodeId,
}

impl BPlusTree {
    pub fn new(order: usize) -> BPlusTree {
        BPlusTree {
            order,
            nodes: vec![Node {
                id: 0,
                is_leaf: true,
                parent_id: None,
                childs: vec![],
                inodes: vec![],
            }],
            root_id: 0,
        }
    }
    pub fn add(&mut self, key: Key, value: Vec<u8>) {
        let target_node_id = self._search(&key);
        self.insert_key_to_node(target_node_id, key, Some(value));

        let mut node_to_split = Some(target_node_id);
        while let Some(node_id) = node_to_split {
            if self.node(node_id).inodes.len() < self.order {
                break;
            }

            self.split(node_id);
            node_to_split = self.node(node_id).parent_id;
        }
    }

    pub fn get(&self, key: Key) -> Option<&Vec<u8>> {
        let target_node = self.node(self._search(&key));

        for inode in &target_node.inodes {
            if inode.key == key {
                return inode.value.as_ref();
            }
        }

        return None;
    }

    fn node_mut(&mut self, id: NodeId) -> &mut Node {
        &mut self.nodes[id as usize]
    }

    fn node(&self, id: NodeId) -> &Node {
        &self.nodes[id as usize]
    }

    fn insert_key_to_node(&mut self, node_id: NodeId, key: Key, value: Option<Vec<u8>>) {
        let ret_idx = self.node_mut(node_id).inodes.binary_search_by_key(&key, |inode| inode.key)
            .unwrap_or_else(|x| x);

        self.node_mut(node_id).inodes.insert(ret_idx, INode { key, value });
    }

    // Регистрирует ноду в дереве и обновляет ссылки у дочерних элементов на вновь созданный ID
    fn create_node(&mut self, is_leaf: bool, parent_id: Option<NodeId>, inodes: Vec<INode>, childs: Vec<NodeId>) -> NodeId {
        let id = self.nodes.len() as NodeId;
        for &child_id in childs.iter() {
            self.node_mut(child_id).parent_id = Some(id);
        }

        self.nodes.push(Node {
            id: self.nodes.len() as NodeId,
            is_leaf,
            parent_id,
            childs,
            inodes,
        });

        return id;
    }

    fn split(&mut self, left_node_id: NodeId) {
        let middle = self.order / 2;

        // Правая нода забирает себе старшие ключи и потомков, которые
        // содержат старшие диапазоны (если это не лист)
        let is_leaf = self.node(left_node_id).is_leaf;
        let right_inodes = self.node_mut(left_node_id).inodes.split_off(middle);
        let right_childs = if self.node_mut(left_node_id).childs.len() > 0 {
            self.node_mut(left_node_id).childs.split_off(middle + 1)
        } else {
            Vec::<NodeId>::new()
        };

        let parent_id = self.node_mut(left_node_id).parent_id;
        let right_node_id = self.create_node(is_leaf, parent_id, right_inodes, right_childs);

        // Если делим родительский элемент, то первый элемент правого поддерева уходит его предку
        // и в правой ноде он становится вообще бесполезен.
        let first_right_key = self.node_mut(right_node_id).inodes[0].key;
        if self.node(right_node_id).childs.len() > 0 {
            self.node_mut(right_node_id).inodes.remove(0);
        }

        if let Some(parent_id) = parent_id {
            // Добавляем наименьший ключ нового узла родителю
            self.insert_key_to_node(parent_id, first_right_key, None);

            // Вставляем ссылку на новый узел сразу же после исходного узла
            let parent = self.node_mut(parent_id);

            let right_child_idx = parent.childs.iter()
                .position(|&x| x == left_node_id)
                .expect("Invalid parent_id on node. Node not found in parent.childs");

            parent.childs.insert(right_child_idx + 1, right_node_id);
        } else {
            // Расщепляется корень, надо создать новый
            self.root_id = self.create_node(
                false,
                None,
                vec![INode { key: first_right_key, value: None }],
                vec![left_node_id, right_node_id],
            );
        };
    }

    fn _search(&self, key: &Key) -> NodeId {
        return self._tree_search(key, self.root_id);
    }

    fn _tree_search(&self, key: &Key, node_id: NodeId) -> NodeId {
        let node = self.node(node_id);
        if node.childs.len() == 0 {
            return node_id;
        }

        // Ищем индекс ребенка, в котором искать дальше:
        // key < keys[0] -> 0
        // keys[i] <= key < key[i+1] -> i
        // key > keys[last] -> last
        let child_index = node.inodes.iter()
            .position(|x| key < &x.key)
            .unwrap_or(node.childs.len() - 1);

        return self._tree_search(key, node.childs[child_index]);
    }
}

fn str_to_key(val: &str) -> Key {
    let mut ret: Key = [0; MAX_KEY_SIZE];
    for (i, &b) in val.as_bytes().iter().rev().enumerate() {
        ret[MAX_KEY_SIZE - i - 1] = b;
    }

    ret
}

fn key_to_str(val: &Key) -> String {
    String::from_iter(
        val.iter()
            .filter(|&&x| x != 0)
            .map(|&x| x as char)
    )
}

impl Display for BPlusTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut stack = Vec::<(NodeId, u32)>::new();
        stack.push((self.root_id, 0));

        loop {
            if stack.len() == 0 {
                break;
            }

            let (node_id, level) = stack.pop().unwrap();

            let node = &self.nodes[node_id as usize];

            let mut name: String = String::new();
            for _ in 0..level {
                name.push_str("  ");
                if level > 0 {
                    name.push_str("|");
                }
            }
            name.push_str("--");

            for (idx, k) in node.inodes.iter().enumerate() {
                name.push_str(key_to_str(&k.key).as_str());

                if idx < node.inodes.len() - 1 {
                    name.push_str(",");
                }
            }

            write!(f, "{} (id={}, parent_id={:?})\n", name.as_str(), node.id, node.parent_id)?;

            for &child_id in node.childs.iter() {
                stack.push((child_id, level + 1));
            }
        }

        fmt::Result::Ok(())
    }
}

pub fn val_to_str(val: Option<&Vec<u8>>) -> &str {
    if val.is_none() {
        return "None";
    }

    return str::from_utf8(val.unwrap()).unwrap();
}

// extern crate serde_derive;
extern crate bincode;

use std::fs::{File, OpenOptions};
use memmap::{Mmap, MmapOptions};
use std::io::{Write, Result};
use serde::{Serialize, Deserialize};
use std::ptr::slice_from_raw_parts;
use std::mem::size_of;
use std::os::unix::fs::FileExt;
use crate::types::{Page, LeafStoredINode, BranchStoredINode, PAGE_BRANCH};

const VERSION: u32 = 1;
const MAGIC: u32 = 0x9B9AB9EE;


fn to_bytes<T>(val: &T) -> &[u8] where T: Sized {
    let raw_h: *const u8 = (val as *const T) as *const u8;

    unsafe {
        slice_from_raw_parts(raw_h, size_of::<T>() as usize).as_ref().unwrap()
    }
}

struct Allocator {
    page_size: usize,
    free_pages: Vec::<u64>,
    // page_ids
    allocated_pages: Vec::<u64>,
}

impl Allocator {
    pub fn new(page_size: usize, file_len: u64) -> Allocator {
        let page_count = (file_len / page_size as u64) as usize;

        let mut free_pages = vec![];
        for i in 0..page_count {
            free_pages.push(i as u64);
        }

        Allocator {
            page_size,
            free_pages,
            allocated_pages: vec![],
        }
    }

    pub fn get_free_page(&mut self, size: u64) -> Option<Page> {
        let mut total_size = 0 as u64;
        let mut pages = Vec::<u64>::new();
        loop {
            if self.free_pages.is_empty() {
                return None;
            }

            let page_id = self.free_pages.remove(0);
            pages.push(page_id);
            self.allocated_pages.push(page_id);
            total_size += self.page_size as u64;

            if total_size >= size {
                break;
            }
        }

        Some(Page {
            id: pages[0],
            page_overflow_count: (pages.len() - 1) as u32,
            flags: 0,
            inode_count: 0,
        })
    }
}

fn from_bytes<T>(buf: &mut [u8]) -> Option<&mut T> where T: Sized {
    let (_, mut body, _) = unsafe { buf.align_to_mut::<T>() };

    if body.len() == 1 {
        return Some(&mut body[0]);
    }

    None
}

fn serialize_data<T>(buf: &mut Vec<u8>, offset: usize, data: T) -> usize where T: Sized {
    let data_page: &mut T = from_bytes(buf[offset..offset + size_of::<T>()].as_mut()).unwrap();
    *data_page = data;

    offset + size_of::<T>()
}

fn save_tree(tree: &BPlusTree, path: &str) -> std::io::Result<()> {
    let page_size = page_size::get();
    let mut f = OpenOptions::new().read(true).write(true).create(true).open(path)?;
    f.set_len(0)?;
    f.set_len((page_size * 100) as u64)?;

    let h = types::Meta {
        magic: MAGIC,
        version: VERSION,
        page_size: page_size as u32,
    };

    let mut allocator = Allocator::new(page_size, f.metadata().unwrap().len());

    let mut page = allocator.get_free_page(page_size as u64).unwrap();
    page.flags = types::PAGE_META;

    f.write(to_bytes(&page))?;
    f.write(to_bytes(&h))?;

    let mut stack = vec![tree.root_id];
    loop {
        if stack.is_empty() {
            break;
        }

        let node_id = stack.pop().unwrap();
        let node_size = tree.node(node_id).size();
        let mut node_data = Vec::<u8>::new();
        node_data.resize(node_size as usize, 0);

        let mut page = allocator.get_free_page(node_size).unwrap();
        page.inode_count = tree.node(node_id).inodes.len() as u32;
        page.flags = PAGE_BRANCH;

        let page_id = page.id;
        println!("{:?}", page);

        let mut offset: usize = 0;
        {
            offset = serialize_data(&mut node_data, offset, page);
        }

        for inode in tree.node(node_id).inodes.iter() {
            let data: BranchStoredINode = BranchStoredINode {
                pos: offset as u32,
                ksize: inode.key.len() as u32,
                page_id,
            };

            offset = serialize_data(&mut node_data, offset, data);

            node_data[offset..offset+inode.key.len()].copy_from_slice(inode.key.as_ref());
            offset += inode.key.len();
            // offset = serialize_data(&mut node_data, offset, inode.key.as_ref());
        }


        f.write_at(node_data.as_slice(), page_id * page_size as u64).unwrap();
    }

//    f.write_at();

    Ok(())
}

fn main() {
    let mut tree = BPlusTree::new(4);
    tree.add(str_to_key("1"), "asd1".bytes().collect());
    tree.add(str_to_key("2"), "asd2".bytes().collect());
    tree.add(str_to_key("3"), "asd3".bytes().collect());
    tree.add(str_to_key("4"), "asd4".bytes().collect());
    tree.add(str_to_key("5"), "asd5".bytes().collect());
    tree.add(str_to_key("6"), "asd6".bytes().collect());
    tree.add(str_to_key("7"), "asd7".bytes().collect());
    tree.add(str_to_key("8"), "asd8".bytes().collect());
    tree.add(str_to_key("9"), "asd9".bytes().collect());
    tree.add(str_to_key("10"), "asd10".bytes().collect());
    tree.add(str_to_key("11"), "asd11".bytes().collect());
    tree.add(str_to_key("12"), "asd12".bytes().collect());
    tree.add(str_to_key("13"), "asd13".bytes().collect());
    tree.add(str_to_key("14"), "asd14".bytes().collect());
    tree.add(str_to_key("15"), "asd15".bytes().collect());
    tree.add(str_to_key("16"), "asd16".bytes().collect());

    tree.add(str_to_key("88"), "asd88".bytes().collect());
    tree.add(str_to_key("56"), "asd56".bytes().collect());
    tree.add(str_to_key("100"), "asd100".bytes().collect());
    tree.add(str_to_key("33"), "asd33".bytes().collect());
    tree.add(str_to_key("54"), "asd54".bytes().collect());
    tree.add(str_to_key("65"), "asd65".bytes().collect());
    tree.add(str_to_key("41"), "asd41".bytes().collect());
    tree.add(str_to_key("24"), "asd24".bytes().collect());
    tree.add(str_to_key("92"), "asd92".bytes().collect());

    println!("{}", &tree);
    println!("{}", val_to_str(tree.get(str_to_key("1"))));
    save_tree(&tree, "/home/anton/workspace/rust-bplustree-example/db.rust").unwrap();
}
