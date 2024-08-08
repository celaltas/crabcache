use crate::{
    entry::Entry,
    hashtable::{fnv1a_hash, HashNode},
    scalablehashmap::ScalableHashMap,
    serialization::{response_nil, response_string},
};
use anyhow::Result;
use container_of::container_of;

pub fn invoke(db: &mut ScalableHashMap, key: Vec<u8>, out: &mut Vec<u8>) -> Result<()> {
    let code = fnv1a_hash(&key);
    let node = HashNode::new(None, code);
    let entry = Entry::new(node, key, None);
    let cmp = Entry::check_entry_equality;
    let found = db.lookup(&entry.node, cmp);
    if found.is_none() {
        // response_nil(out);
        let message = "not found";
        response_string(out, message.as_bytes());
        return Ok(());
    }
    let found = found.unwrap();
    let entry = unsafe {
        let found = found as *const HashNode;
        let e = container_of!(found, Entry, node);
        &*e
    };
    if let Some(value) = &entry.value {
        response_string(out, value);
        return Ok(());
    }

    response_nil(out);
    return Ok(());
}
