use std::cmp::Ordering;

type Key = [u8; 32];

type NodeId = i64;
const INVALID_NODE_ID: NodeId = -1;

// https://gist.github.com/savarin/69acd246302567395f65ad6b97ee503d

struct BPlusTree {
    order: usize, // Параметр дерева

    nodes: Vec<Node>,
    root_id: NodeId,
}

impl BPlusTree {
    pub fn add(&mut self, key: Key) {
        let node_id = self._search(&key);
        self.insert_key_to_node(node_id, key);

        if self.nodes[node_id as usize].keys.len() >= self.order {
            let mut node_to_split = node_id;
            while node_to_split != INVALID_NODE_ID && self.nodes[node_to_split as usize].keys.len() >= self.order {
                self.split(node_to_split);
                node_to_split = self.nodes[node_to_split as usize].parent_id;
            }
        }
    }

    fn insert_key_to_node(&mut self, node_id: NodeId, key: Key) {
        let mut ret_idx = None;
        for (idx, i) in self.nodes[node_id as usize].keys.iter().enumerate() {
            // key < i
            if key < *i {
                ret_idx = Some(idx);
                break;
            }
        }

        if let Some(idx) = ret_idx {
            self.nodes[node_id as usize].keys.insert(idx, key);
        } else {
            self.nodes[node_id as usize].keys.push(key);
        }
    }

    fn split(&mut self, node_id: NodeId) {
        let middle = self.order / 2;
        let right_node_id = self.nodes.len() as NodeId;

        let parent_id = self.nodes[node_id as usize].parent_id;
        let right_keys = {
            let node = &mut self.nodes[node_id as usize];

            node.keys.split_off(middle)
        };

        let right_childs = {
            let node = &mut self.nodes[node_id as usize];

            if node.childs.len() > 0 {
                node.childs.split_off(middle + 1)
            } else {
                Vec::<NodeId>::new()
            }
        };

        for child_id in right_childs.iter() {
            self.nodes[*child_id as usize].parent_id = right_node_id;
        }

        self.nodes.push(Node {
            id: right_node_id,
            parent_id: INVALID_NODE_ID,
            keys: right_keys,
            childs: right_childs,
        });

        let new_parent_id = {
            if parent_id == INVALID_NODE_ID {
                // Расщепляется корень, надо создать новый
                let new_root_id = self.nodes.len() as NodeId;
                let mut new_parent_node = Node {
                    id: new_root_id,
                    parent_id: INVALID_NODE_ID,
                    keys: vec![self.nodes[right_node_id as usize].keys[0]],
                    childs: vec![node_id, right_node_id],
                };

                self.nodes.push(new_parent_node);
                self.root_id = new_root_id;

                new_root_id
            } else {
                let new_key = self.nodes[right_node_id as usize].keys[0];
                // let parent = &mut self.nodes[parent_id as usize];

                // FIXME: search correct position
                self.insert_key_to_node(parent_id, new_key);

                let parent = &mut self.nodes[parent_id as usize];
                let mut idx = None;
                for (i, child_id) in parent.childs.iter().enumerate() {
                    if node_id == *child_id {
                        idx = Some(i);
                        break;
                    }
                }

                if let Some(i) = idx {
                    parent.childs.insert(i + 1, right_node_id);
                } else {
                    panic!("Invalid parent");
                }
                // self.insert_child_to_node(parent_id, right_node_id);
                // parent.childs.push(right_node_id);

                parent_id
            }
        };

        // Если длим родительский элемент, то первый элемент правого поддерева уходит его предку
        // и в правой ноде он становится вообще бесполезен.
        if self.nodes[node_id as usize].childs.len() > 0 {
            self.nodes[right_node_id as usize].keys.remove(0);
        }

        self.nodes[node_id as usize].parent_id = new_parent_id;
        self.nodes[right_node_id as usize].parent_id = new_parent_id;
    }

    fn _search(&self, key: &Key) -> NodeId {
        return self._tree_search(key, self.root_id);
    }

    fn _tree_search(&self, key: &Key, node_id: NodeId) -> NodeId {
        let node = &self.nodes[node_id as usize];
        if node.childs.len() == 0 {
            return node_id;
        }

        // Ищем индекс ребенка, в котором искать дальше:
        // key < keys[0] -> 0
        // keys[i] <= key < key[i+1] -> i
        // key > keys[last] -> last
        let mut child_index = INVALID_NODE_ID;
        for (idx, range_key) in self.nodes[node_id as usize].keys.iter().enumerate() {
            if key < range_key {
                child_index = idx as NodeId;
                break;
            }
        }

        if child_index == INVALID_NODE_ID {
            child_index = (self.nodes[node_id as usize].childs.len() - 1) as NodeId;
        }

        return self._tree_search(key, node.childs[child_index as usize]);
    }
}

fn str_to_key(val: &str) -> Key {
    let mut ret: Key = [0; 32];
    for (i, b) in val.as_bytes().iter().enumerate() {
        ret[32 - val.len() + i] = *b;
    }

    return ret;
}

fn key_to_str(val: &Key) -> String {
    let mut name: String = String::new();

    for i in val.iter() {
        if *i == 0 {
            continue
        }

        name.push(*i as char);
    };

    return name;
}

struct Node {
    id: NodeId,
    parent_id: NodeId,
    keys: Vec<Key>,
    childs: Vec<NodeId>,
}

fn print_tree(tree: &BPlusTree) {
    let mut stack = Vec::<(NodeId, u32)>::new();
    stack.push((tree.root_id, 0));

    loop {
        if stack.len() == 0 {
            break;
        }

        let (node_id, level) = stack.pop().unwrap();

        let node = &tree.nodes[node_id as usize];

        let mut name: String = String::new();
        for i in 0..level {
            name.push_str("  ");
        }

        if level > 0 {
            name.push_str("| ");
        }

        for k in node.keys.iter() {
            name.push_str(key_to_str(k).as_str());
            name.push_str(",");
        }

        println!("{} (id={}, parent_id={})", name.as_str(), node.id, node.parent_id);

        for child_id in node.childs.iter() {
            stack.push((*child_id, level + 1));
        }
    }
}

fn main() {
    let mut qwe = vec![1,2,3];
    qwe.insert(qwe.len() -1 , 4);
    println!("{:?}", qwe);
    let mut tree = BPlusTree {
        order: 4,

        nodes: Vec::new(),
        root_id: 0,
    };

    tree.nodes.push(Node {
        id: 0,
        parent_id: INVALID_NODE_ID,
        keys: Vec::<Key>::new(),
        childs: Vec::<NodeId>::new(),
    });

    tree.add(str_to_key("1"));
    tree.add(str_to_key("2"));
    tree.add(str_to_key("3"));
    tree.add(str_to_key("4"));
    tree.add(str_to_key("5"));
    tree.add(str_to_key("6"));
    tree.add(str_to_key("7"));
    tree.add(str_to_key("8"));
    tree.add(str_to_key("9"));
    tree.add(str_to_key("10"));
    tree.add(str_to_key("11"));
    tree.add(str_to_key("12"));
    tree.add(str_to_key("13"));
    tree.add(str_to_key("14"));
    tree.add(str_to_key("15"));
    tree.add(str_to_key("16"));


    tree.add(str_to_key("88"));
    tree.add(str_to_key("56"));
    tree.add(str_to_key("100"));
    tree.add(str_to_key("33"));
    tree.add(str_to_key("54"));
    tree.add(str_to_key("65"));
    tree.add(str_to_key("41"));
    tree.add(str_to_key("24"));
    tree.add(str_to_key("92"));

    //    let s = std::str::from_utf8(&tree.nodes[0].keys[3]).unwrap();
    //    println!("{}", s);

    print_tree(&tree);

    println!("{:?}", &str_to_key("40") > &str_to_key("094"));
}
