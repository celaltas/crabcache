use std::{fmt::Display, ops::RangeBounds};

use anyhow::Result;

#[derive(Debug, Clone)]
pub struct HashNode {
    next: Option<Box<HashNode>>,
    code: u64,
}

impl PartialEq for HashNode {
    fn eq(&self, other: &Self) -> bool {
        self.code == other.code
    }
}

impl HashNode {
    pub fn new(next: Option<HashNode>, code: u64) -> Self {
        if let Some(n) = next {
            Self {
                next: Some(Box::new(n)),
                code,
            }
        } else {
            Self { next: None, code }
        }
    }
   
}

pub fn fnv1a_hash(bytes: &[u8]) -> u64 {
    let mut h: u32 = 0x811C9DC5;
    for &byte in bytes {
        h = h.wrapping_add(byte as u32).wrapping_mul(0x01000193);
    }
    h as u64
}

#[derive(Debug)]
pub struct HashTable {
    pub table: Vec<Option<Box<HashNode>>>,
    size: usize,
    mask: usize,
}

impl Display for HashTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "HashTable {{ size: {}, mask: {} }} \n table: {:#?}",
            self.size, self.mask, self.table
        )
    }
}

impl HashTable {
    pub fn new(n: usize) -> Result<HashTable> {
        anyhow::ensure!(n.is_power_of_two(), "Size must be a power of 2");
        let mask = n - 1;
        let mut table = Vec::with_capacity(n);
        table.resize_with(n, || None);
        let size = 0;
        Ok(Self { table, size, mask })
    }

    pub fn insert(&mut self, node: &mut HashNode) {
        let pos = (node.code & (self.mask as u64)) as usize;
        let next = self.table.get_mut(pos).and_then(Option::take);
        if let Some(n) = next {
            node.next = Some(n);
        }
        self.table[pos] = Some(Box::new(node.clone()));
        self.size += 1;
    }

    pub fn lookup(
        &self,
        node: &HashNode,
        cmp: fn(&HashNode, &HashNode) -> bool,
    ) -> Option<&HashNode> {
        if self.table.is_empty() {
            return None;
        }
        let pos = (node.code & (self.mask as u64)) as usize;
        let mut from = self.table[pos].as_ref();
        while let Some(n) = from {
            let address_of_found_node = &**n as *const HashNode;
            println!("lookup:\n exist:{:#?}\n, search (re): {:#?}\n, address_of_found_node: {:#?}\n", n, node, address_of_found_node);
            if cmp(n, node) {
                return Some(n);
            }
            from = n.next.as_ref();
        }
        None
    }
    pub fn lookup_mut(
        &mut self,
        node: &HashNode,
        cmp: fn(&HashNode, &HashNode) -> bool,
    ) -> Option<&mut HashNode> {
        if self.table.is_empty() {
            return None;
        }
        let pos = (node.code & (self.mask as u64)) as usize;
        let mut from = self.table[pos].as_mut();
        while let Some(n) = from {
            if cmp(n, node) {
                return Some(n);
            }
            from = n.next.as_mut();
        }
        None
    }

    pub fn detach(&mut self, from: &mut HashNode) -> Option<HashNode> {
        let _ = std::mem::replace(&mut from.next, None);
        self.size -= 1;
        Some(HashNode {
            code: from.code,
            next: None,
        })
    }

    pub fn drain<R>(&mut self, range: R) -> impl Iterator<Item = Box<HashNode>> + '_
    where
        R: RangeBounds<usize>,
    {
        self.table.drain(range).filter_map(|node| {
            if node.is_some() {
                self.size -= 1;
            }
            node
        })
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn mask(&self) -> usize {
        self.mask
    }

    pub fn resize(&mut self, new_size: usize) {
        self.table.resize(new_size, None);
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    fn print_out_linked_list(head: &Option<Box<HashNode>>) {
        let mut current = head;
        while let Some(node) = current {
            print!("{}->", node.code);
            current = &node.next;
        }
    }

    fn print_out_hash_table(ht: &HashTable) {
        for i in 0..ht.table.len() {
            if ht.table[i].is_none() {
                continue;
            }
            println!("\n");
            print_out_linked_list(&ht.table[i]);
        }
    }

    fn generate_node_list(n: usize) -> Vec<HashNode> {
        let mut nodes = Vec::with_capacity(n);
        for i in 0..n {
            nodes.push(HashNode {
                next: None,
                code: i as u64,
            });
        }
        nodes
    }

    fn generate_linked_list(n: usize) -> Option<Box<HashNode>> {
        let mut head = None;
        let mut tail = &mut head;
        for i in 0..n {
            *tail = Some(Box::new(HashNode {
                next: None,
                code: i as u64,
            }));
            tail = &mut (*tail).as_mut().unwrap().next;
        }
        head
    }

    #[test]
    fn test_new() {
        let ht = HashTable::new(1024);
        assert!(ht.is_ok());
        let ht = HashTable::new(10);
        assert!(ht.is_err());
        let ht = HashTable::new(0);
        assert!(ht.is_err());
        let ht = HashTable::new(1);
        assert!(ht.is_ok());
    }

    #[test]
    fn test_insert() {
        let mut ht = HashTable::new(4).unwrap();
        let mut nodes = generate_node_list(18);
        for n in nodes.iter_mut() {
            ht.insert(n);
        }
        assert_eq!(ht.size, 18);
        print_out_hash_table(&ht)
    }

    #[test]
    fn test_lookup() {
        let mut ht = HashTable::new(4).unwrap();

        let mut nodes = generate_node_list(18);
        for n in nodes.iter_mut() {
            ht.insert(n);
        }
        print_out_hash_table(&ht);
        println!("\n\n");
        let node1 = Box::new(HashNode {
            next: None,
            code: 4,
        });

        let found = ht.lookup(&node1, |a, b| a == b);
        assert!(found.is_some());
        assert_eq!(found.unwrap().code, 4);
        println!("node1 found!");

        let node2 = Box::new(HashNode {
            next: None,
            code: 13,
        });

        let found = ht.lookup(&node2, |a, b| a == b);
        assert!(found.is_some());
        assert_eq!(found.unwrap().code, 13);
        println!("node2 found!");

        let node3 = Box::new(HashNode {
            next: None,
            code: 21,
        });

        let found = ht.lookup(&node3, |a, b| a == b);
        assert!(found.is_none());
        println!("node3 not found!");

        let node4 = Box::new(HashNode {
            next: None,
            code: 45,
        });
        let found = ht.lookup(&node4, |a, b| a == b);
        assert!(found.is_none());
        println!("node4 not found!");

        let node5 = Box::new(HashNode {
            next: None,
            code: 16,
        });

        let found = ht.lookup(&node5, |a, b| a == b);
        assert!(found.is_some());
        assert_eq!(found.unwrap().code, 16);
        println!("node5 found!");
        println!("found: {:#?}", found.unwrap())
    }

    #[test]
    fn test_detach() {
        let mut ht = HashTable::new(8).unwrap();
        ht.size = 5;

        let head = generate_linked_list(5);
        print_out_linked_list(&head);

        if let Some(mut first_node) = head {
            let node = first_node.next.as_mut();
            if let Some(node) = node {
                let detached = ht.detach(node);
                println!("Detached node: {:?}", detached);
                assert_eq!(detached.unwrap().code, 1);
            }
        }
    }
}
