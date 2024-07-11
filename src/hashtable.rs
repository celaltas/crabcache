use anyhow::{ensure, Result};

#[derive(Debug, Clone)]
struct HashNode {
    next: Option<Box<HashNode>>,
    code: u64,
}

struct HashTable {
    table: Vec<Option<Box<HashNode>>>,
    size: usize,
    mask: usize,
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

    pub fn insert(&mut self, mut node: Box<HashNode>) {
        let pos = (node.code & (self.mask as u64)) as usize;
        let next = self.table.get_mut(pos).and_then(Option::take);
        if let Some(n) = next {
            node.next = Some(n);
        }
        self.table[pos] = Some(node);
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
            if cmp(n, node) {
                return Some(n);
            }
            from = n.next.as_ref();
        }
        None
    }

    pub fn detach(&mut self, from: &mut Option<Box<HashNode>>) -> Option<Box<HashNode>> {
        if let Some(mut node) = from.take() {
            *from = node.next.take();
            self.size -= 1;
            Some(node)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn print_out_linked_list(head: &Option<Box<HashNode>>) {
        let mut head = head;
        while let Some(node) = head {
            print!("{}->", node.code);
            head = &node.next;
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

    fn generate_node_list(n: usize) -> Vec<Box<HashNode>> {
        let mut nodes = Vec::with_capacity(n);
        for i in 0..n {
            nodes.push(Box::new(HashNode {
                next: None,
                code: i as u64,
            }));
        }
        nodes
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
        let nodes = generate_node_list(18);
        for n in nodes {
            ht.insert(n);
        }
        assert_eq!(ht.size, 18);
        print_out_hash_table(&ht)
    }

    #[test]
    fn test_lookup() {
        let mut ht = HashTable::new(4).unwrap();

        let nodes = generate_node_list(18);
        for n in nodes {
            ht.insert(n);
        }
        print_out_hash_table(&ht);
        println!("\n\n");
        let node1 = Box::new(HashNode {
            next: None,
            code: 4,
        });

        let found = ht.lookup(&node1, |a, b| a.code == b.code);
        assert!(found.is_some());
        assert_eq!(found.unwrap().code, 4);
        println!("node1 found!");

        let node2 = Box::new(HashNode {
            next: None,
            code: 13,
        });

        let found = ht.lookup(&node2, |a, b| a.code == b.code);
        assert!(found.is_some());
        assert_eq!(found.unwrap().code, 13);
        println!("node2 found!");

        let node3 = Box::new(HashNode {
            next: None,
            code: 21,
        });

        let found = ht.lookup(&node3, |a, b| a.code == b.code);
        assert!(found.is_none());
        println!("node3 not found!");

        let node4 = Box::new(HashNode {
            next: None,
            code: 45,
        });
        let found = ht.lookup(&node4, |a, b| a.code == b.code);
        assert!(found.is_none());
        println!("node4 not found!");


        let node5 = Box::new(HashNode {
            next: None,
            code: 16,
        });

        let found = ht.lookup(&node5, |a, b| a.code == b.code);
        assert!(found.is_some());
        assert_eq!(found.unwrap().code, 16);
        println!("node5 found!");
        println!("found: {:#?}", found.unwrap())

        
    }
}
