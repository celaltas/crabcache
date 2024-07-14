use anyhow::ensure;

use crate::hashtable::{HashNode, HashTable};

const LOAD_FACTOR: usize = 8;
const RESIZING_WORK: usize = 128;

struct ScalableHashMap {
    table1: Option<HashTable>,
    table2: Option<HashTable>,
    resizing_pos: usize,
}

impl ScalableHashMap {
    pub fn new() -> ScalableHashMap {
        ScalableHashMap {
            table1: Some(HashTable::new(4).unwrap()),
            table2: None,
            resizing_pos: 0,
        }
    }

    pub fn start_resizing(&mut self) {
        assert!(self.table2.is_none());
        let capacity = self.table1.as_ref().map_or(4, |t| (t.mask() + 1) * 2);
        self.table2 = self.table1.take();
        self.table1 = Some(HashTable::new(capacity).unwrap());
        self.resizing_pos = 0;
    }

    fn help_resizing(&mut self) {
        if self.table2.is_none() {
            return;
        }

        let mut nwork = 0;
        let size = self.table2.as_ref().map_or(0, |t| t.size());
        while nwork < RESIZING_WORK && size > 0 {
            let resizing_pos = self.resizing_pos;
            let node = &mut self.table2.unwrap().table[resizing_pos];
            if node.is_none() {
                self.resizing_pos += 1;
                continue;
            }
            let node = node.as_mut().unwrap();
            let detached = self.table2.as_mut().map_or(None, |t| t.detach(node));

            if let Some(n) = detached {
                self.table1.as_mut().map(|table| table.insert(n));
                nwork += 1;
            }
        }

        if self.table2.as_ref().map_or(0, |t| t.size()) == 0 {
            self.table2 = None;
        }
    }

    pub fn lookup(
        &self,
        key: &HashNode,
        cmp: fn(&HashNode, &HashNode) -> bool,
    ) -> Option<&HashNode> {
        //help resize
        self.table1
            .as_ref()
            .and_then(|t| t.lookup(key, cmp))
            .or_else(|| self.table2.as_ref().and_then(|t| t.lookup(key, cmp)))
    }

    pub fn insert(&mut self, node: HashNode) {
        if let Some(table) = self.table1.as_mut() {
            table.insert(node)
        } else {
            self.table1 = Some(HashTable::new(4).unwrap());
        }
        if self.table2.is_none() {
            let length = self.table1.as_ref().unwrap().size();
            let mask = self.table1.as_ref().unwrap().mask() + 1;
            let load_factor = length / mask;
            if load_factor >= LOAD_FACTOR {
                println!("resize");
                //start resize
            }
        }
        //help resize
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
        self.resizing_pos = 0;
    }
}
