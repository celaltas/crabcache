use crate::{
    entry::Entry,
    hashtable::{fnv1a_hash, HashNode},
    scalablehashmap::ScalableHashMap,
    serialization::response_integer,
};
use anyhow::Result;
use container_of::container_of;

pub fn invoke(db: &mut ScalableHashMap, key: Vec<u8>, out: &mut Vec<u8>) -> Result<()> {
    let code = fnv1a_hash(&key);
    let node = HashNode::new(None, code);
    let mut entry = Entry::new(node, key, None);
    let cmp = Entry::check_entry_equality;
    let found = db.pop(&mut entry.node, cmp);
    if found.is_some() {
        let found = &found.unwrap();
        let entry = unsafe {
            let found = found as *const HashNode;
            let e = container_of!(found, Entry, node);
            &*e
        };
        drop(entry);
        response_integer(out, 1);
        return Ok(());
    } else {
        response_integer(out, 0);
        return Ok(());
    }
}
