use crate::{
    entry::Entry,
    hashtable::{fnv1a_hash, HashNode},
    scalablehashmap::ScalableHashMap,
    serialization::response_nil,
};
use anyhow::Result;
use container_of::container_of;

pub fn invoke(db: &mut ScalableHashMap, key: Vec<u8>, value: Vec<u8>) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    let code = fnv1a_hash(&key);
    let key_for_not_found = key.clone();
    let node = HashNode::new(None, code);
    let entry = Entry::new(node, key, Some(value.clone()));
    let address_of_entry = &entry as *const Entry;
    let address_of_node = &entry.node as *const HashNode;
    println!(
        "the entry created: {:#?}\naddress of entry: {:#?}\naddress of node: {:#?}\n",
        entry, address_of_entry, address_of_node
    );
    let cmp = Entry::check_entry_equality;
    let found = db.lookup_mut(&entry.node, cmp);
    if found.is_none() {
        let node = HashNode::new(None, code);
        let mut entry = Entry::new(node, key_for_not_found, Some(value));
        db.insert(&mut entry.node);
        response_nil(&mut out);
        return Ok(out);
    }
    let found = found.unwrap();

    let entry = unsafe {
        let found = found as *mut HashNode;
        let e = container_of!(found, Entry, node);
        &mut *e
    };

    entry.value = Some(value);
    response_nil(&mut out);
    Ok(out)
}
