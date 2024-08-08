use crate::{hashtable::HashNode, scalablehashmap::ScalableHashMap};
use container_of::container_of;

#[repr(C)]
#[derive(Debug)]

pub struct Entry {
    pub node: HashNode,
    pub key: Vec<u8>,
    pub value: Option<Vec<u8>>,
}

impl Entry {
    pub fn new(node: HashNode, key: Vec<u8>, value: Option<Vec<u8>>) -> Self {
        Self { node, key, value }
    }

    pub fn check_entry_equality(left: &HashNode, right: &HashNode) -> bool {
        let le = unsafe {
            let left = left as *const HashNode;
            container_of!(left, Entry, node)
        };
        let re = unsafe {
            let right = right as *const HashNode;
            container_of!(right, Entry, node)
        };
        let entry_left = unsafe { &*le };
        let entry_right = unsafe { &*re };
        entry_left.key == entry_right.key
    }

}

struct Data {
    db: ScalableHashMap,
}

impl Data {
    pub fn new() -> Self {
        Self {
            db: ScalableHashMap::new(),
        }
    }
}
