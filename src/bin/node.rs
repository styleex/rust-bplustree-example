use crate::types::{PageId, PageHeader};

const MAX_KEY_SIZE: usize = 32;

type Key = [u8; MAX_KEY_SIZE];
type NodeId = usize;

pub enum INodes<'a> {
    Mmaped {key: &'a [u8], value: Option<&'a [u8]>},
    Runtime {key: Vec<u8>, value: Option<Vec<u8>>}
}


// Для листа содержит и ключ и значение. Для родителя только ключи
#[repr(C, packed)]
pub(crate) struct INode<'a> {
    pub(crate) key: &'a [u8],
    pub(crate) value: Option<&'a [u8]>,

    pub(crate) key_vec: Option<Vec<u8>>,
    pub(crate) value_vec: Option<Vec<u8>>,

    page_id: Option<PageId>,
}

// https://gist.github.com/savarin/69acd246302567395f65ad6b97ee503d
#[repr(C, packed)]
pub struct Node<'a> {
    id: NodeId,
    is_leaf: bool,
    parent_id: Option<NodeId>,
    childs: Vec<NodeId>,
    page_id: PageId,

    // runtime only
    pub(crate) inodes: Vec<INode<'a>>,
}


pub struct NodeCache<'a> {
    pub(crate) nodes: Vec<Node<'a>>,
}

impl<'a> NodeCache<'a> {
    pub fn new() -> NodeCache<'a> {
        NodeCache {
            nodes: vec![],
        }
    }

    pub fn read_node(&mut self, p: &'a PageHeader) -> NodeId {
        let mut inodes = Vec::<INode>::new();

        for idx in 0..(p.inode_count as usize) {
            if p.is_leaf() {
                inodes.push(INode {
                    key: p.leaf_inodes()[idx].key(),
                    value: Some(p.leaf_inodes()[idx].value()),
                    value_vec: None,
                    page_id: None,
                });
            } else {
                inodes.push(INode {
                    key: p.branch_inodes()[idx].key(),
                    value: None,
                    value_vec: None,
                    page_id: Some(p.branch_inodes()[idx].page_id as PageId),
                });
            }
        }

        let id = self.nodes.len();
        self.nodes.push(Node {
            id,
            is_leaf: p.is_leaf(),
            parent_id: None,
            childs: vec![],
            page_id: p.id,
            inodes,
        });

        id
    }
}
