use crate::{
    entry::Entry,
    hashtable::{fnv1a_hash, HashNode},
    scalablehashmap::ScalableHashMap, serialization::{response_nil, response_string},
};
use anyhow::Result;
use container_of::container_of;

pub fn invoke(db: &mut ScalableHashMap, key: Vec<u8>) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    let code = fnv1a_hash(&key);
    let node = HashNode::new(None, code);
    let entry = Entry::new(node, key, None);
    let cmp = Entry::check_entry_equality;
    let found = db.lookup(&entry.node, cmp);
    if found.is_none() {
        response_nil(&mut out);
        return Ok(out);
    }
    let found = found.unwrap();
    let entry = unsafe {
        let found = found as *const HashNode;
        let e = container_of!(found, Entry, node);
        &*e
    };
    let message = "OK";
    //response string should accept byte array
    response_string(&mut out, message);
    Ok(out)
}
