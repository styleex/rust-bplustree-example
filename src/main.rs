use std::rc::{Rc, Weak};
use std::ops::DerefMut;
use core::borrow::BorrowMut;
use std::cell::RefCell;

type Key = [u8; 32];
type NodeId = usize;

struct BPlusTree {
    t: u32,
    n: u32,

    nodes: Vec<Node>,
}

impl BPlusTree {
    pub fn add(&mut self, key: Key) {
        let node_id = self._search(key);

        self.nodes[node_id].keys.push(key)
    }

    fn _search(&self, key: Key) -> NodeId {
        return self._tree_search(key, 0);
    }

    fn _tree_search(&self, key: Key, node_id: NodeId) -> NodeId {
        let node = &self.nodes[node_id];
        if node.childs.len() == 0 {
            return node_id;
        }

        return self._tree_search(key, 0);
    }
}

fn str_to_key(val: &str) -> Key {
    let b = val.as_bytes();

    let mut ret: Key = [0; 32];
    for (i, b) in val.as_bytes().iter().enumerate() {
        ret[i] = *b;
    }

    return ret;
}

struct Node {
    keys: Vec<Key>,
    childs: Vec<NodeId>,
}


fn print_tree(tree: &BPlusTree) {
    let mut stack = Vec::<(NodeId, u32)>::new();
    stack.push((0, 0));


    loop {
        if stack.len() == 0 {
            break;
        }

        let (node_id, level) = stack.pop().unwrap();

        let node = &tree.nodes[node_id];

        let mut name: String = String::new();
        for k in node.keys.iter() {
            name.push_str(std::str::from_utf8(k).unwrap());
            name.push_str(",");
        }

        println!("{}", name.as_str());

        for child_id in node.childs.iter() {
            stack.push((*child_id, level + 1));
        }
    }
}

fn main() {
    let mut tree = BPlusTree {
        n: 3,
        t: 3,

        nodes: Vec::new(),
    };

    tree.nodes.push(Node {
        keys: Vec::<Key>::new(),
        childs: Vec::<NodeId>::new(),
    });

    tree.add(str_to_key("1"));
    tree.add(str_to_key("2"));
    tree.add(str_to_key("3"));
    tree.add(str_to_key("4"));
    tree.add(str_to_key("5"));
    tree.add(str_to_key("6"));

    tree.nodes.push(Node {
        keys: Vec::<Key>::new(),
        childs: Vec::<NodeId>::new(),
    });
    tree.nodes[0].childs.push(1);
    tree.nodes[1].keys.push(str_to_key("6"));

//    let s = std::str::from_utf8(&tree.nodes[0].keys[3]).unwrap();
//    println!("{}", s);

    print_tree(&tree)
}
