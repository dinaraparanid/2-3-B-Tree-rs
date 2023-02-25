mod btree {
    use std::{
        cell::RefCell,
        cmp::Ordering,
        hint::unreachable_unchecked,
        rc::{Rc, Weak},
    };

    const MAX_KEYS: usize = 2;
    const MAX_CHILDREN: usize = 3;

    #[derive(Debug)]
    enum BTreeNode<T: Ord + Eq + Clone> {
        Leaf { leaf: BTreeLeaf<T> },
        SubTree { subtree: BTreeSubTree<T> },
    }

    #[derive(Debug, Default)]
    struct BTreeLeaf<T: Ord + Eq + Clone> {
        values: Vec<Rc<T>>,
        parent: Option<Weak<RefCell<BTreeNode<T>>>>,
        next_leaf: Option<Rc<RefCell<BTreeNode<T>>>>,
        previous_leaf: Option<Weak<RefCell<BTreeNode<T>>>>,
    }

    #[derive(Debug, Default)]
    struct BTreeSubTree<T: Ord + Eq + Clone> {
        children: Vec<Rc<RefCell<BTreeNode<T>>>>,
        parent: Option<Weak<RefCell<BTreeNode<T>>>>,
        mid_keys: Vec<Rc<T>>,
    }

    #[derive(Debug, Default)]
    pub struct BTree<T: Ord + Eq + Clone> {
        root: Option<Rc<RefCell<BTreeNode<T>>>>,
        size: usize,
    }

    impl<T: Ord + Eq + Clone> BTreeLeaf<T> {
        #[inline]
        pub fn new(
            values: Vec<Rc<T>>,
            parent: Option<Weak<RefCell<BTreeNode<T>>>>,
            next_leaf: Option<Rc<RefCell<BTreeNode<T>>>>,
            previous_leaf: Option<Weak<RefCell<BTreeNode<T>>>>,
        ) -> Self {
            Self {
                values,
                parent,
                next_leaf,
                previous_leaf,
            }
        }
    }

    impl<T: Ord + Eq + Clone> BTreeSubTree<T> {
        #[inline]
        pub fn new(
            children: Vec<Rc<RefCell<BTreeNode<T>>>>,
            parent: Option<Weak<RefCell<BTreeNode<T>>>>,
            mid_keys: Vec<Rc<T>>,
        ) -> Self {
            Self {
                children,
                parent,
                mid_keys,
            }
        }
    }

    impl<T: Ord + Eq + Clone> BTreeNode<T> {
        #[inline]
        pub fn is_leaf(&self) -> bool {
            match self {
                BTreeNode::Leaf { .. } => true,
                BTreeNode::SubTree { .. } => false,
            }
        }

        #[inline]
        pub fn is_node(&self) -> bool {
            !self.is_leaf()
        }

        #[inline]
        pub fn unwrap_as_leaf(&self) -> &BTreeLeaf<T> {
            match self {
                BTreeNode::Leaf { leaf } => leaf,
                BTreeNode::SubTree { .. } => unreachable!(),
            }
        }

        #[inline]
        pub fn unwrap_as_leaf_mut(&mut self) -> &mut BTreeLeaf<T> {
            match self {
                BTreeNode::Leaf { leaf } => leaf,
                BTreeNode::SubTree { .. } => unreachable!(),
            }
        }

        #[inline]
        pub unsafe fn unwrap_as_leaf_unchecked(&self) -> &BTreeLeaf<T> {
            match self {
                BTreeNode::Leaf { leaf } => leaf,
                BTreeNode::SubTree { .. } => unreachable_unchecked(),
            }
        }

        #[inline]
        pub unsafe fn unwrap_as_leaf_mut_unchecked(&mut self) -> &mut BTreeLeaf<T> {
            match self {
                BTreeNode::Leaf { leaf } => leaf,
                BTreeNode::SubTree { .. } => unreachable_unchecked(),
            }
        }

        #[inline]
        pub fn unwrap_as_subtree(&self) -> &BTreeSubTree<T> {
            match self {
                BTreeNode::SubTree { subtree } => subtree,
                BTreeNode::Leaf { .. } => unreachable!(),
            }
        }

        #[inline]
        pub unsafe fn unwrap_as_subtree_unchecked(&self) -> &BTreeSubTree<T> {
            match self {
                BTreeNode::SubTree { subtree } => subtree,
                BTreeNode::Leaf { .. } => unreachable_unchecked(),
            }
        }

        #[inline]
        pub fn unwrap_as_subtree_mut(&mut self) -> &mut BTreeSubTree<T> {
            match self {
                BTreeNode::SubTree { subtree } => subtree,
                BTreeNode::Leaf { .. } => unreachable!(),
            }
        }

        #[inline]
        pub unsafe fn unwrap_as_subtree_mut_unchecked(&mut self) -> &mut BTreeSubTree<T> {
            match self {
                BTreeNode::SubTree { subtree } => subtree,
                BTreeNode::Leaf { .. } => unreachable_unchecked(),
            }
        }
    }

    impl<T: Ord + Eq + Clone> BTree<T> {
        #[inline]
        pub const fn new() -> Self {
            Self {
                root: None,
                size: 0,
            }
        }

        #[inline]
        pub fn insert(&mut self, value: T) {
            self.size += 1;

            match &mut self.root {
                None => {
                    self.root = Some(Rc::new(RefCell::new(BTreeNode::Leaf {
                        leaf: BTreeLeaf::new(value, None, None, None),
                    })));
                }

                Some(node) => match &mut node.borrow_mut() {
                    BTreeNode::Leaf { .. } => self.insert_to_root_leaf(value),
                    BTreeNode::SubTree { subtree } => self.insert_to_subtree(subtree, value),
                },
            }
        }

        #[inline]
        fn insert_to_root_leaf(&mut self, value: T) {
            let mut leaf = unsafe {
                self.root
                    .unwrap()
                    .borrow_mut()
                    .unwrap_as_leaf_mut_unchecked()
            };

            leaf.values.push(value);
            leaf.values.sort_by(|a, b| a.cmp(&*b));

            if leaf.values.len() <= MAX_KEYS {
                return;
            }

            let first_leaf = Rc::new(RefCell::new(BTreeNode::Leaf {
                leaf: BTreeLeaf::new(vec![leaf.values.first().unwrap().clone()], None, None, None),
            }));

            let second_leaf = Rc::new(RefCell::new(BTreeNode::Leaf {
                leaf: BTreeLeaf::new(
                    leaf.values[1..].iter().map(|x| x.clone()).collect(),
                    None,
                    None,
                    Some(Rc::downgrade(&first_leaf)),
                ),
            }));

            unsafe {
                first_leaf
                    .borrow_mut()
                    .unwrap_as_leaf_mut_unchecked()
                    .next_leaf = Some(second_leaf.clone())
            }

            let mut new_root = Rc::new(RefCell::new(BTreeNode::SubTree {
                subtree: BTreeSubTree::new(
                    vec![first_leaf, second_leaf],
                    None,
                    vec![leaf.values[1].clone()],
                ),
            }));

            unsafe {
                first_leaf
                    .borrow_mut()
                    .unwrap_as_leaf_mut_unchecked()
                    .parent = Some(Rc::downgrade(&new_root));

                second_leaf
                    .borrow_mut()
                    .unwrap_as_leaf_mut_unchecked()
                    .parent = Some(Rc::downgrade(&new_root))
            }

            self.root = Some(new_root)
        }

        #[inline]
        fn insert_to_subtree(&mut self, subtree: &mut BTreeSubTree<T>, value: T) {
            let child_subtree_index = match subtree.children.len() {
                1 => match value.cmp(&*subtree.mid_keys[0]) {
                    Ordering::Less => 0,
                    _ => 1,
                },

                2 => {
                    if value < *subtree.mid_keys[0] {
                        0
                    } else if value > *subtree.mid_keys[0] && value < *subtree.mid_keys[1] {
                        1
                    } else {
                        2
                    }
                }

                _ => unreachable!(),
            };

            self.insert_to_children_subtree(subtree, child_subtree_index, value)
        }

        #[inline]
        fn insert_to_children_subtree(
            &mut self,
            subtree: &mut BTreeSubTree<T>,
            child_subtree_index: usize,
            value: T,
        ) {
            match &mut *subtree.children[child_subtree_index].borrow_mut() {
                BTreeNode::Leaf { leaf } => self.insert_to_leaf(leaf, child_subtree_index, value),
                BTreeNode::SubTree { subtree } => self.insert_to_subtree(subtree, value),
            }
        }

        #[inline]
        fn insert_to_leaf(&mut self, leaf: &mut BTreeLeaf<T>, leaf_ind: usize, value: T) {
            leaf.values.push(value);
            leaf.values.sort();

            if leaf.values.len() <= MAX_KEYS {
                return;
            }

            let first_leaf = Rc::new(RefCell::new(BTreeNode::Leaf {
                leaf: BTreeLeaf::new(
                    vec![leaf.values.first().unwrap().clone()],
                    leaf.parent.clone(),
                    None,
                    leaf.previous_leaf.clone(),
                ),
            }));

            let second_leaf = Rc::new(RefCell::new(BTreeNode::Leaf {
                leaf: BTreeLeaf::new(
                    leaf.values[1..].iter().map(|x| x.clone()).collect(),
                    leaf.parent.clone(),
                    leaf.next_leaf.clone(),
                    Some(Rc::downgrade(&first_leaf)),
                ),
            }));

            unsafe {
                first_leaf
                    .borrow_mut()
                    .unwrap_as_leaf_mut_unchecked()
                    .next_leaf = Some(second_leaf.clone())
            }

            let parent_tree = leaf.parent.unwrap().upgrade().unwrap().clone();

            unsafe {
                let parent_tree = leaf
                    .parent
                    .unwrap()
                    .upgrade()
                    .unwrap()
                    .clone()
                    .borrow_mut()
                    .unwrap_as_subtree_mut_unchecked();

                parent_tree.children.remove(leaf_ind);
                parent_tree.children.push(first_leaf);
                parent_tree.children.push(second_leaf);
            };

            self.insert_mid_key_to_parent_subtree(parent_tree, leaf.values[1].clone())
        }

        fn insert_mid_key_to_parent_subtree(
            &mut self,
            subtree: Rc<RefCell<BTreeNode<T>>>,
            mid_key: Rc<T>,
        ) {
            unsafe {
                let tree = subtree.borrow_mut().unwrap_as_subtree_mut_unchecked();

                tree.mid_keys.push(mid_key);
                tree.mid_keys.sort_by(|a, b| a.cmp(&*b));

                if tree.mid_keys.len() <= MAX_KEYS {
                    return;
                }
            }

            match unsafe {
                &mut subtree
                    .borrow_mut()
                    .unwrap_as_subtree_mut_unchecked()
                    .parent
                    .map(|node| node.upgrade().unwrap())
            } {
                None => self.rebalance_root_after_mid_key_insertion(),

                Some(parent_tree) => unsafe {
                    let tree = subtree.borrow_mut().unwrap_as_subtree_mut_unchecked();

                    let first_subtree = Rc::new(RefCell::new(BTreeNode::SubTree {
                        subtree: BTreeSubTree::new(
                            tree.children[..2].iter().map(|x| x.clone()).collect(),
                            Some(Rc::downgrade(&subtree)),
                            tree.mid_keys[..2].iter().map(|x| x.clone()).collect(),
                        ),
                    }));

                    let second_subtree = Rc::new(RefCell::new(BTreeNode::SubTree {
                        subtree: BTreeSubTree::new(
                            tree.children[2..].iter().map(|x| x.clone()).collect(),
                            Some(Rc::downgrade(&subtree)),
                            tree.mid_keys[2..].iter().map(|x| x.clone()).collect(),
                        ),
                    }));

                    let parent_tree_ref =
                        parent_tree.borrow_mut().unwrap_as_subtree_mut_unchecked();

                    parent_tree_ref.children.push(first_subtree);
                    parent_tree_ref.children.push(second_subtree);

                    self.insert_mid_key_to_parent_subtree(
                        parent_tree.clone(),
                        tree.mid_keys[1].clone(),
                    );
                },
            }
        }

        #[inline]
        fn rebalance_root_after_mid_key_insertion(&mut self) {
            let mut root_tree = unsafe {
                self.root
                    .unwrap()
                    .borrow_mut()
                    .unwrap_as_subtree_mut_unchecked()
            };

            let first_subtree = Rc::new(RefCell::new(BTreeNode::SubTree {
                subtree: BTreeSubTree::new(
                    root_tree.children[..2].iter().map(|x| x.clone()).collect(),
                    None,
                    root_tree.mid_keys[..2].iter().map(|x| x.clone()).collect(),
                ),
            }));

            let second_subtree = Rc::new(RefCell::new(BTreeNode::SubTree {
                subtree: BTreeSubTree::new(
                    root_tree.children[2..].iter().map(|x| x.clone()).collect(),
                    None,
                    root_tree.mid_keys[2..].iter().map(|x| x.clone()).collect(),
                ),
            }));

            let mut new_root = Rc::new(RefCell::new(BTreeNode::SubTree {
                subtree: BTreeSubTree::new(
                    vec![first_subtree, second_subtree],
                    None,
                    vec![root_tree.mid_keys[1].clone()],
                ),
            }));

            unsafe {
                first_subtree
                    .borrow_mut()
                    .unwrap_as_leaf_mut_unchecked()
                    .parent = Some(Rc::downgrade(&new_root));

                second_subtree
                    .borrow_mut()
                    .unwrap_as_leaf_mut_unchecked()
                    .parent = Some(Rc::downgrade(&new_root))
            }

            self.root = Some(new_root)
        }
    }
}

fn main() {}
