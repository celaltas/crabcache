use crate::{
    entry::Entry,
    hashtable::{fnv1a_hash, HashNode},
    scalablehashmap::ScalableHashMap,
    serialization::response_nil,
};
use anyhow::Result;
use container_of::container_of;

pub fn invoke(
    db: &mut ScalableHashMap,
    key: Vec<u8>,
    value: Vec<u8>,
    out: &mut Vec<u8>,
) -> Result<()> {
    let code = fnv1a_hash(&key);
    let key_for_not_found = key.clone();
    let node = HashNode::new(None, code);
    let entry = Entry::new(node, key, Some(value.clone()));
    let cmp = Entry::check_entry_equality;
    let found = db.lookup_mut(&entry.node, cmp);
    if found.is_none() {
        let node = HashNode::new(None, code);
        let mut entry = Entry::new(node, key_for_not_found, Some(value));
        db.insert(&mut entry.node);
        response_nil(out);
        return Ok(());
    }
    let found = found.unwrap();

    let entry = unsafe {
        let found = found as *mut HashNode;
        let e = container_of!(found, Entry, node);
        &mut *e
    };

    entry.value = Some(value);
    response_nil(out);
    Ok(())
}
