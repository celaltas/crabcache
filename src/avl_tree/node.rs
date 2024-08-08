use std::{cmp::max, fmt::Debug};
pub type AvlTree<T> = Option<Box<AvlNode<T>>>;


#[derive(Debug)]
pub enum AVLNodeError {
    RotateError(RotateError),
    NodeNotFound,
}

#[derive(Debug)]
pub enum RotateError {
    NoLeftChild,
    NoRightChild,
}

#[derive(Debug, PartialEq, Clone)]
pub struct AvlNode<T: Ord> {
    pub value: T,
    pub height: usize,
    pub left: AvlTree<T>,
    pub right: AvlTree<T>,
}

impl<T: Ord> AvlNode<T> {
    pub fn left_height(&self) -> usize {
        self.left.as_ref().map_or(0, |l| l.height)
    }
    pub fn right_height(&self) -> usize {
        self.right.as_ref().map_or(0, |l| l.height)
    }
    pub fn update_height(&mut self) {
        self.height = 1 + max(self.left_height(), self.right_height());
    }

    pub fn balance_factor(&self) -> i8 {
        self.left_height() as i8 - self.right_height() as i8
    }

    pub fn rotate_right(&mut self) -> Result<(), RotateError> {
        let mut left_node = self.left.take().ok_or(RotateError::NoLeftChild)?;
        self.left = left_node.right.take();
        self.update_height();
        std::mem::swap(self, &mut left_node);
        self.right = Some(left_node);
        self.update_height();
        Ok(())
    }

    pub fn rotate_left(&mut self) -> Result<(), RotateError> {
        let mut right_node = self.right.take().ok_or(RotateError::NoRightChild)?;
        self.right = right_node.left.take();
        self.update_height();
        std::mem::swap(self, right_node.as_mut());
        self.left = Some(right_node);
        self.update_height();
        Ok(())
    }

    pub fn rebalance(&mut self) -> Result<(), AVLNodeError> {
        match self.balance_factor() {
            -2 => {
                let right_node = self.right.as_mut().ok_or(AVLNodeError::NodeNotFound)?;
                if right_node.balance_factor() == 1 {
                    right_node
                        .rotate_right()
                        .map_err(AVLNodeError::RotateError)?;
                }
                self.rotate_left().map_err(AVLNodeError::RotateError)?;
                Ok(())
            }
            2 => {
                let left_node = self.left.as_mut().ok_or(AVLNodeError::NodeNotFound)?;
                if left_node.balance_factor() == -1 {
                    left_node.rotate_left().map_err(AVLNodeError::RotateError)?;
                }
                self.rotate_right().map_err(AVLNodeError::RotateError)?;
                Ok(())
            }
            _ => Ok({}),
        }
    }
}
