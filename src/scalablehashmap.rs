use std::fmt::Display;
use crate::hashtable::{HashNode, HashTable};

const LOAD_FACTOR: usize = 8;

pub struct ScalableHashMap {
    table1: Option<HashTable>,
    table2: Option<HashTable>,
}

impl Display for ScalableHashMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ScalableHashMap {{\n")?;
        write!(f, " table1: {:#?}\n", &self.table1)?;
        write!(f, " table2: {:#?}\n", &self.table2)?;
        write!(f, " }}\n")
    }
}

impl ScalableHashMap {
    pub fn new() -> ScalableHashMap {
        ScalableHashMap {
            table1: Some(HashTable::new(4).unwrap()),
            table2: None,
        }
    }

    pub fn start_resizing(&mut self) {
        assert!(self.table2.is_none());
        let capacity = self.table1.as_ref().map_or(4, |t| (t.mask() + 1) * 2);
        self.table2 = self.table1.take();
        self.table1 = Some(HashTable::new(capacity).unwrap());
        self.table1.as_mut().unwrap().resize(capacity);
    }

    fn help_resizing(&mut self) {
        if self.table2.is_none() {
            return;
        }
        if let (Some(noble), Some(substitute)) = (&mut self.table2, &mut self.table1) {
            noble.drain(..noble.size()).for_each(|mut item| {
                substitute.insert(&mut item);
            });
        }
        if self.table2.as_ref().map_or(0, |t| t.size()) == 0 {
            self.table2 = None;
        }
    }

    pub fn lookup(
        &mut self,
        key: &HashNode,
        cmp: fn(&HashNode, &HashNode) -> bool,
    ) -> Option<&HashNode> {
        self.help_resizing();
        self.table1
            .as_ref()
            .and_then(|t| t.lookup(key, cmp))
            .or_else(|| self.table2.as_ref().and_then(|t| t.lookup(key, cmp)))
    }
    pub fn lookup_mut(
        &mut self,
        key: &HashNode,
        cmp: fn(&HashNode, &HashNode) -> bool,
    ) -> Option<&mut HashNode> {
        self.help_resizing();
        self.table1
            .as_mut()
            .and_then(|t| t.lookup_mut(key, cmp))
            .or_else(|| self.table2.as_mut().and_then(|t| t.lookup_mut(key, cmp)))
    }

    pub fn insert(&mut self, node: &mut HashNode) {
        if let Some(table) = self.table1.as_mut() {
            table.insert(node)
        } else {
            self.table1 = Some(HashTable::new(4).unwrap());
        }
        if self.table2.is_none() {
            let length = self.table1.as_ref().unwrap().size();
            let mask = self.table1.as_ref().unwrap().mask() + 1;
            let loaded = length / mask;
            if loaded >= LOAD_FACTOR {
                self.start_resizing();
            }
        }
        self.help_resizing()
    }

    fn pop_from_table(
        table: &mut Option<HashTable>,
        node: &mut HashNode,
        cmp: fn(&HashNode, &HashNode) -> bool,
    ) -> Option<HashNode> {
        table.as_mut().and_then(|t| {
            if t.lookup(node, cmp).is_some() {
                t.detach(node)
            } else {
                None
            }
        })
    }

    pub fn pop(
        &mut self,
        node: &mut HashNode,
        cmp: fn(&HashNode, &HashNode) -> bool,
    ) -> Option<HashNode> {
        self.help_resizing();
        let mut table1 = self.table1.take();
        let found = Self::pop_from_table(&mut table1, node, cmp);
        self.table1 = table1;
        if found.is_none() {
            let mut table2 = self.table2.take();
            let found = Self::pop_from_table(&mut table2, node, cmp);
            self.table2 = table2;
            return found;
        }
        return found;
    }

    pub fn size(&self) -> usize {
        let size1 = self.table1.as_ref().map_or(0, |t| t.size());
        let size2 = self.table2.as_ref().map_or(0, |t| t.size());
        size1 + size2
    }

    pub fn destroy(&mut self) {
        self.table1 = None;
        self.table2 = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn generate_node_list(n: usize) -> Vec<HashNode> {
        let mut list = Vec::new();
        for i in 0..n {
            list.push(HashNode::new(None, i.try_into().unwrap()));
        }
        list
    }

    #[test]
    fn test_new() {
        let map = ScalableHashMap::new();
        assert!(map.table1.is_some());
        assert!(map.table2.is_none());
        assert_eq!(map.size(), 0)
    }

    #[test]
    fn test_insert() {
        let mut map = ScalableHashMap::new();
        let node_number = 15;
        let mut nodes = generate_node_list(node_number);
        for node in nodes.iter_mut() {
            map.insert(node);
        }
        assert_eq!(map.size(), node_number);
    }

    #[test]
    fn test_lookup() {
        let mut map = ScalableHashMap::new();
        let node_number = 15;
        let mut nodes = generate_node_list(node_number);
        let nodes_for_search = nodes.clone();

        for node in nodes.iter_mut() {
            map.insert(node);
        }
        for node in &nodes_for_search {
            assert_eq!(map.lookup(node, |a, b| a == b), Some(node));
        }
        assert_eq!(map.lookup(&HashNode::new(None, 123), |a, b| a == b), None);
        assert_eq!(map.lookup(&HashNode::new(None, 101), |a, b| a == b), None);
    }

    #[test]
    fn test_pop() {
        let mut map = ScalableHashMap::new();
        let node_number = 15;
        let mut nodes = generate_node_list(node_number);
        let nodes_for_search = nodes.clone();

        for node in nodes.iter_mut() {
            map.insert(node);
        }
        for mut node in nodes_for_search {
            assert_eq!(map.pop(&mut node, |a, b| a == b), Some(node));
        }
        assert_eq!(map.pop(&mut HashNode::new(None, 123), |a, b| a == b), None);
        assert_eq!(map.pop(&mut HashNode::new(None, 101), |a, b| a == b), None);
    }

    #[test]
    fn test_start_resize() {
        let mut map = ScalableHashMap::new();
        let node_number = 3;
        let mut nodes = generate_node_list(node_number);
        for node in nodes.iter_mut() {
            map.insert(node);
        }
        map.start_resizing();
        assert_eq!(map.size(), 3);
        let node = HashNode::new(None, 2);
        assert_eq!(map.lookup(&node, |a, b| a == b), Some(&node));
    }

    #[test]
    fn test_help_resizing() {
        let mut map = ScalableHashMap::new();
        let node_number = 3;
        let mut nodes = generate_node_list(node_number);
        for node in nodes.iter_mut() {
            map.insert(node);
        }
        map.start_resizing();
        assert_eq!(map.size(), 3);
        let node = HashNode::new(None, 2);
        assert_eq!(map.lookup(&node, |a, b| a == b), Some(&node));
        map.help_resizing();
        assert_eq!(map.size(), 3);
        assert!(map.table2.is_none());
    }
}
