use std::fmt::Display;
use core::fmt;

const MAX_KEY_SIZE: usize = 32;

type Key = [u8; MAX_KEY_SIZE];

type NodeId = i64;

// https://gist.github.com/savarin/69acd246302567395f65ad6b97ee503d

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
                parent_id: None,
                childs: vec![],
                keys: vec![],
            }],
            root_id: 0,
        }
    }
    pub fn add(&mut self, key: Key) {
        let target_node_id = self._search(&key);
        self.insert_key_to_node(target_node_id, key);

        let mut node_to_split = Some(target_node_id);
        while let Some(node_id) = node_to_split {
            if self.node(node_id).keys.len() < self.order {
                break;
            }

            self.split(node_id);
            node_to_split = self.node(node_id).parent_id;
        }
    }

    fn node_mut(&mut self, id: NodeId) -> &mut Node {
        &mut self.nodes[id as usize]
    }

    fn node(&self, id: NodeId) -> &Node {
        &self.nodes[id as usize]
    }

    fn insert_key_to_node(&mut self, node_id: NodeId, key: Key) {
        let ret_idx = self.node_mut(node_id).keys.binary_search(&key)
            .unwrap_or_else(|x| x);

        self.node_mut(node_id).keys.insert(ret_idx, key);
    }

    // Регистрирует ноду в дереве и обновляет ссылки у дочерних элементов на вновь созданный ID
    fn create_node(&mut self, parent_id: Option<NodeId>, keys: Vec<Key>, childs: Vec<NodeId>) -> NodeId {
        let id = self.nodes.len() as NodeId;
        for &child_id in childs.iter() {
            self.node_mut(child_id).parent_id = Some(id);
        }

        self.nodes.push(Node {
            id: self.nodes.len() as NodeId,
            parent_id,
            keys,
            childs,
        });

        return id;
    }

    fn split(&mut self, left_node_id: NodeId) {
        let middle = self.order / 2;

        // Правая нода забирает себе старшие ключи и потомков, которые
        // содержат старшие диапазоны (если это не лист)
        let right_keys = self.node_mut(left_node_id).keys.split_off(middle);
        let right_childs = if self.node_mut(left_node_id).childs.len() > 0 {
            self.node_mut(left_node_id).childs.split_off(middle + 1)
        } else {
            Vec::<NodeId>::new()
        };

        let parent_id = self.node_mut(left_node_id).parent_id;
        let right_node_id = self.create_node(parent_id, right_keys, right_childs);

        // Если делим родительский элемент, то первый элемент правого поддерева уходит его предку
        // и в правой ноде он становится вообще бесполезен.
        let first_right_key = self.node_mut(right_node_id).keys[0];
        if self.node(right_node_id).childs.len() > 0 {
            self.node_mut(right_node_id).keys.remove(0);
        }

        if let Some(parent_id) = parent_id {
            // Добавляем наименьший ключ нового узла родителю
            self.insert_key_to_node(parent_id, first_right_key);

            // Вставляем ссылку на новый узел сразу же после исходного узла
            let parent = self.node_mut(parent_id);

            let right_child_idx = parent.childs.iter()
                .position(|&x| x == left_node_id)
                .expect("Invalid parent_id on node. Node not found in parent.childs");

            parent.childs.insert(right_child_idx + 1, right_node_id);
        } else {
            // Расщепляется корень, надо создать новый
            self.root_id = self.create_node(
                None,
                vec![first_right_key],
                vec![left_node_id, right_node_id]);
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
        let child_index = node.keys.iter()
            .position(|x| key < x)
            .unwrap_or(node.childs.len() - 1);

        return self._tree_search(key, node.childs[child_index as usize]);
    }
}

fn str_to_key(val: &str) -> Key {
    let mut ret: Key = [0; MAX_KEY_SIZE];
    for (i, &b) in val.as_bytes().iter().rev().enumerate() {
        ret[MAX_KEY_SIZE - i - 1] = b;
    }

    return ret;
}

fn key_to_str(val: &Key) -> String {
    let mut name: String = String::new();

    for &i in val.iter() {
        if i == 0 {
            continue;
        }

        name.push(i as char);
    };

    return name;
}

struct Node {
    id: NodeId,
    parent_id: Option<NodeId>,
    keys: Vec<Key>,
    childs: Vec<NodeId>,
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
            }

            if level > 0 {
                name.push_str("| ");
            }

            for k in node.keys.iter() {
                name.push_str(key_to_str(k).as_str());
                name.push_str(",");
            }

            write!(f, "{} (id={}, parent_id={:?})\n", name.as_str(), node.id, node.parent_id)?;

            for &child_id in node.childs.iter() {
                stack.push((child_id, level + 1));
            }
        }

        fmt::Result::Ok(())
    }
}
fn main() {
    let mut tree = BPlusTree::new(4);
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

    println!("{}", &tree);
}
