use quickcheck::{Arbitrary, Gen};
use std::{cmp::Ordering, fmt::Debug, mem::swap};
use super::node::{AVLNodeError, AvlNode, AvlTree};

#[derive(Debug)]
pub enum AvlTreeSetError {
    InsertError(InsertError),
}

#[derive(Debug)]
pub enum InsertError {
    AvlError(AVLNodeError),
    AlreadyExistError,
}

#[derive(Debug)]
pub enum DeleteError {
    AvlError(AVLNodeError),
    NodeNotFound,
}

#[derive(Debug, PartialEq, Clone)]
struct AvlTreeSet<T: Ord> {
    root: AvlTree<T>,
}

impl<'a, T: 'a + Ord + Debug> AvlTreeSet<T> {
    pub fn new() -> Self {
        Self { root: None }
    }

    pub fn delete(&mut self, value: &T) -> Result<(), DeleteError> {
        let mut path = Vec::<*mut AvlNode<T>>::new();
        let mut current = &mut self.root;

        let target = loop {
            if let Some(node) = current {
                match value.cmp(&node.value) {
                    Ordering::Equal => break &mut **node,
                    Ordering::Less => {
                        path.push(&mut **node);
                        current = &mut node.left;
                    }
                    Ordering::Greater => {
                        path.push(&mut **node);
                        current = &mut node.right;
                    }
                }
            } else {
                return Err(DeleteError::NodeNotFound);
            }
        };

        match (target.left.is_some(), target.right.is_some()) {
            (false, false) => {
                if let Some(parent) = path.last() {
                    let parent = unsafe { &mut **parent };
                    if parent
                        .left
                        .as_ref()
                        .map_or(false, |left_node| target.value == left_node.value)
                    {
                        parent.left = None
                    } else {
                        parent.right = None
                    }
                } else {
                    self.root = None
                }
            }
            (true, false) | (false, true) => {
                let mut child = target.left.take().or_else(|| target.right.take()).unwrap();
                swap(target, &mut *child);
            }
            (true, true) => {
                let right_tree = &mut target.right;
                if right_tree
                    .as_ref()
                    .map_or(false, |node| node.left.is_none())
                {
                    let mut right_node = right_tree.take().unwrap();
                    right_node.left = target.left.take();
                    right_node.right = target.right.take();
                    swap(target, &mut right_node);
                } else {
                    let mut next_tree = right_tree;
                    let mut inner_path = Vec::<*mut AvlNode<T>>::new();
                    while let Some(next_left_node) = next_tree {
                        if next_left_node.left.is_some() {
                            inner_path.push(&mut **next_left_node)
                        }
                        next_tree = &mut next_left_node.left;
                    }
                    let parent_left_node = unsafe { &mut *inner_path.pop().unwrap() };
                    let mut leftmost_node = parent_left_node.left.take().unwrap();

                    leftmost_node.left = target.left.take();
                    leftmost_node.right = target.right.take();

                    swap(target, &mut leftmost_node);
                    swap(&mut parent_left_node.left, &mut leftmost_node.right);

                    parent_left_node.update_height();
                    parent_left_node
                        .rebalance()
                        .map_err(DeleteError::AvlError)?;

                    Self::rebalance_path(&inner_path)?;
                }
                target.update_height();
                target.rebalance().map_err(DeleteError::AvlError)?;
            }
        };

        Self::rebalance_path(&path)
    }

    fn rebalance_path(path: &[*mut AvlNode<T>]) -> Result<(), DeleteError> {
        for node_ptr in path {
            let node = unsafe { &mut **node_ptr };
            node.update_height();
            node.rebalance().map_err(DeleteError::AvlError)?;
        }

        Ok(())
    }

    pub fn insert(&mut self, value: T) -> Result<(), InsertError> {
        let mut prev_ptrs = Vec::<*mut AvlNode<T>>::new();

        let mut current_tree = &mut self.root;
        while let Some(current_node) = current_tree {
            prev_ptrs.push(&mut **current_node);
            match current_node.value.cmp(&value) {
                Ordering::Less => current_tree = &mut current_node.right,
                Ordering::Greater => current_tree = &mut current_node.left,
                Ordering::Equal => return Err(InsertError::AlreadyExistError),
            }
        }
        *current_tree = Some(Box::new(AvlNode {
            value,
            height: 1,
            left: None,
            right: None,
        }));

        for node_ptr in prev_ptrs.into_iter().rev() {
            let node = unsafe { &mut *node_ptr };
            node.update_height();
            node.rebalance().map_err(InsertError::AvlError)?;
        }

        Ok(())
    }

    pub fn iter(&'a self) -> impl Iterator<Item = &'a T> + 'a {
        self.node_iter().map(|node| &node.value)
    }

    fn node_iter(&'a self) -> impl Iterator<Item = &'a AvlNode<T>> + 'a {
        AvlTreeSetIter {
            prev_nodes: Vec::new(),
            current_tree: &self.root,
        }
    }
}

#[cfg(test)]
impl<T: Arbitrary + Ord + Debug> Arbitrary for AvlTreeSet<T> {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        let vec: Vec<T> = Arbitrary::arbitrary(g);
        vec.into_iter().collect()
    }
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        let vec: Vec<T> = self.iter().cloned().collect();
        Box::new(vec.shrink().map(|v| v.into_iter().collect::<Self>()))
    }
}

impl<T: Ord + Debug> FromIterator<T> for AvlTreeSet<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut set = Self::new();
        for i in iter {
            let _ = set.insert(i);
        }
        set
    }
}

struct AvlTreeSetIter<'a, T: Ord> {
    prev_nodes: Vec<&'a AvlNode<T>>,
    current_tree: &'a AvlTree<T>,
}

impl<'a, T: 'a + Ord> Iterator for AvlTreeSetIter<'a, T> {
    type Item = &'a AvlNode<T>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match *self.current_tree {
                None => match self.prev_nodes.pop() {
                    None => {
                        return None;
                    }

                    Some(ref prev_node) => {
                        self.current_tree = &prev_node.right;
                        return Some(prev_node);
                    }
                },

                Some(ref current_node) => {
                    if current_node.left.is_some() {
                        self.prev_nodes.push(&current_node);
                        self.current_tree = &current_node.left;
                        continue;
                    }

                    if current_node.right.is_some() {
                        self.current_tree = &current_node.right;
                        return Some(current_node);
                    }

                    self.current_tree = &None;
                    return Some(current_node);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use std::{cmp::max, collections::BTreeSet};

    use itertools::{all, equal};
    use quickcheck::TestResult;
    use quickcheck_macros::quickcheck;

    use super::*;

    #[quickcheck]
    fn test_node_height(set: AvlTreeSet<u16>) -> bool {
        all(set.node_iter(), |node| {
            node.height == 1 + max(node.left_height(), node.right_height())
        })
    }

    #[quickcheck]
    fn rotate_right_preserves_order(btree: BTreeSet<u8>) -> TestResult {
        let set = btree.iter().cloned().collect::<AvlTreeSet<_>>();
        TestResult::from_bool(equal(set.iter(), btree.iter()))
    }

    #[quickcheck]
    fn balanced_nodes(set: AvlTreeSet<u16>) -> bool {
        all(set.node_iter(), |node| node.balance_factor().abs() < 2)
    }

    #[test]
    fn test_delete() {
        let values = vec![20, 10, 30, 5, 15, 25, 35, 3, 13, 33];

        let mut set = values.iter().cloned().collect::<AvlTreeSet<_>>();
        for i in set.iter() {
            print!("{i}->");
        }
        println!("");
        match set.delete(&10) {
            Ok(_) => {
                println!("************deleted**********");
                for i in set.iter() {
                    print!("{i}->");
                }
                println!("");
            }
            Err(err) => println!("error occured {:#?}", err),
        }
    }
}
