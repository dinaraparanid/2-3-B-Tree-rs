use crate::btree::BTree;

mod btree {
    use std::{
        cell::RefCell,
        cmp::Ordering,
        fmt::Debug,
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
        pub fn len(&self) -> usize {
            self.size
        }

        #[inline]
        pub fn insert(&mut self, value: T) {
            self.size += 1;

            match self.root.is_none() {
                true => {
                    self.root = Some(Rc::new(RefCell::new(BTreeNode::Leaf {
                        leaf: BTreeLeaf::new(vec![Rc::new(value)], None, None, None),
                    })));
                }

                false => match {
                    let is_leaf = self.root.as_ref().unwrap().borrow().is_leaf();
                    is_leaf
                } {
                    true => self.insert_to_root_leaf(value),

                    false => {
                        let subtree = self.root.as_ref().unwrap().clone();
                        self.insert_to_subtree(subtree, value);
                    }
                },
            }
        }

        #[inline]
        fn insert_to_root_leaf(&mut self, value: T) {
            let (first_leaf, second_leaf, mid_key) = unsafe {
                let mut leaf = self.root.as_ref().unwrap().borrow_mut();
                let leaf = leaf.unwrap_as_leaf_mut_unchecked();

                leaf.values.push(Rc::new(value));
                leaf.values.sort_by(|a, b| a.cmp(&*b));

                if leaf.values.len() <= MAX_KEYS {
                    return;
                }

                let first_leaf = Rc::new(RefCell::new(BTreeNode::Leaf {
                    leaf: BTreeLeaf::new(vec![leaf.values[0].clone()], None, None, None),
                }));

                let second_leaf = Rc::new(RefCell::new(BTreeNode::Leaf {
                    leaf: BTreeLeaf::new(
                        leaf.values[1..].iter().map(|x| x.clone()).collect(),
                        None,
                        None,
                        Some(Rc::downgrade(&first_leaf)),
                    ),
                }));

                (
                    first_leaf.clone(),
                    second_leaf.clone(),
                    leaf.values[1].clone(),
                )
            };

            unsafe {
                first_leaf
                    .borrow_mut()
                    .unwrap_as_leaf_mut_unchecked()
                    .next_leaf = Some(second_leaf.clone());
            }

            let new_root = Rc::new(RefCell::new(BTreeNode::SubTree {
                subtree: BTreeSubTree::new(
                    vec![first_leaf.clone(), second_leaf.clone()],
                    None,
                    vec![mid_key],
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
                    .parent = Some(Rc::downgrade(&new_root));
            }

            self.root = Some(new_root)
        }

        #[inline]
        fn insert_to_subtree(&mut self, subtree: Rc<RefCell<BTreeNode<T>>>, value: T) {
            let child_subtree_index = unsafe {
                let subtree_ref = subtree.borrow();
                let subtree_ref = subtree_ref.unwrap_as_subtree_unchecked();

                match subtree_ref.mid_keys.len() {
                    1 => match value.cmp(&*subtree_ref.mid_keys[0]) {
                        Ordering::Less => 0,
                        _ => 1,
                    },

                    2 => {
                        if value < *subtree_ref.mid_keys[0] {
                            0
                        } else if value > *subtree_ref.mid_keys[0]
                            && value < *subtree_ref.mid_keys[1]
                        {
                            1
                        } else {
                            2
                        }
                    }

                    _ => unreachable!(),
                }
            };

            self.insert_to_children_subtree(subtree, child_subtree_index, value)
        }

        #[inline]
        fn insert_to_children_subtree(
            &mut self,
            subtree: Rc<RefCell<BTreeNode<T>>>,
            child_subtree_index: usize,
            value: T,
        ) {
            let node = unsafe {
                let subtree_borrow_ref = subtree.borrow();
                let subtree_ref = subtree_borrow_ref.unwrap_as_subtree_unchecked();
                subtree_ref.children[child_subtree_index].clone()
            };

            match {
                let is_leaf = node.borrow().is_leaf();
                is_leaf
            } {
                true => self.insert_to_leaf(node, child_subtree_index, value),
                false => self.insert_to_subtree(node, value),
            };
        }

        #[inline]
        fn insert_to_leaf(&mut self, leaf: Rc<RefCell<BTreeNode<T>>>, leaf_ind: usize, value: T) {
            unsafe {
                let mut leaf_ref = leaf.borrow_mut();
                let leaf_ref = leaf_ref.unwrap_as_leaf_mut_unchecked();

                leaf_ref.values.push(Rc::new(value));
                leaf_ref.values.sort();

                if leaf_ref.values.len() <= MAX_KEYS {
                    return;
                }

                let first_leaf = Rc::new(RefCell::new(BTreeNode::Leaf {
                    leaf: BTreeLeaf::new(
                        vec![leaf_ref.values[0].clone()],
                        leaf_ref.parent.clone(),
                        None,
                        leaf_ref.previous_leaf.clone(),
                    ),
                }));

                if let Some(prev_leaf) = &leaf_ref.previous_leaf {
                    prev_leaf
                        .upgrade()
                        .unwrap()
                        .borrow_mut()
                        .unwrap_as_leaf_mut_unchecked()
                        .next_leaf = Some(first_leaf.clone());
                }

                let second_leaf = Rc::new(RefCell::new(BTreeNode::Leaf {
                    leaf: BTreeLeaf::new(
                        leaf_ref.values[1..].iter().map(|x| x.clone()).collect(),
                        leaf_ref.parent.clone(),
                        leaf_ref.next_leaf.clone(),
                        Some(Rc::downgrade(&first_leaf)),
                    ),
                }));

                if let Some(next_leaf) = &leaf_ref.next_leaf {
                    next_leaf
                        .borrow_mut()
                        .unwrap_as_leaf_mut_unchecked()
                        .previous_leaf = Some(Rc::downgrade(&second_leaf));
                }

                first_leaf
                    .borrow_mut()
                    .unwrap_as_leaf_mut_unchecked()
                    .next_leaf = Some(second_leaf.clone());

                let parent_tree = leaf_ref.parent.as_ref().unwrap().upgrade().unwrap().clone();

                {
                    let mut parent_subtree = parent_tree.borrow_mut();
                    let parent_subtree = parent_subtree.unwrap_as_subtree_mut_unchecked();
                    parent_subtree.children.remove(leaf_ind);
                    parent_subtree.children.push(first_leaf);
                    parent_subtree.children.push(second_leaf);
                }

                self.insert_mid_key_to_parent_subtree(parent_tree, leaf_ref.values[1].clone())
            }
        }

        fn insert_mid_key_to_parent_subtree(
            &mut self,
            subtree: Rc<RefCell<BTreeNode<T>>>,
            mid_key: Rc<T>,
        ) {
            unsafe {
                let mut tree = subtree.borrow_mut();
                let tree = tree.unwrap_as_subtree_mut_unchecked();

                tree.mid_keys.push(mid_key);
                tree.mid_keys.sort_by(|a, b| a.cmp(&*b));

                if tree.mid_keys.len() <= MAX_KEYS {
                    return;
                }
            }

            match unsafe {
                let is_parent_none = subtree
                    .as_ref()
                    .borrow()
                    .unwrap_as_subtree_unchecked()
                    .parent
                    .is_none();

                is_parent_none
            } {
                true => self.rebalance_root_after_mid_key_insertion(),

                false => {
                    let (first_subtree, second_subtree, mid_key) = unsafe {
                        let mut tree = subtree.borrow_mut();
                        let tree = tree.unwrap_as_subtree_mut_unchecked();

                        let first_subtree = Rc::new(RefCell::new(BTreeNode::SubTree {
                            subtree: BTreeSubTree::new(
                                tree.children[..2].iter().map(|node| node.clone()).collect(),
                                Some(tree.parent.as_ref().unwrap().clone()),
                                vec![tree.mid_keys[0].clone()],
                            ),
                        }));

                        tree.children[..2].iter_mut().for_each(|node| {
                            match &mut *node.borrow_mut() {
                                BTreeNode::Leaf { leaf } => {
                                    leaf.parent = Some(Rc::downgrade(&first_subtree))
                                }

                                BTreeNode::SubTree { subtree } => {
                                    subtree.parent = Some(Rc::downgrade(&first_subtree))
                                }
                            }
                        });

                        let second_subtree = Rc::new(RefCell::new(BTreeNode::SubTree {
                            subtree: BTreeSubTree::new(
                                tree.children[2..].iter().map(|x| x.clone()).collect(),
                                Some(tree.parent.as_ref().unwrap().clone()),
                                vec![tree.mid_keys[2].clone()],
                            ),
                        }));

                        tree.children[2..].iter_mut().for_each(|node| {
                            match &mut *node.borrow_mut() {
                                BTreeNode::Leaf { leaf } => {
                                    leaf.parent = Some(Rc::downgrade(&second_subtree))
                                }

                                BTreeNode::SubTree { subtree } => {
                                    subtree.parent = Some(Rc::downgrade(&second_subtree))
                                }
                            }
                        });

                        (first_subtree, second_subtree, tree.mid_keys[1].clone())
                    };

                    unsafe {
                        let parent_tree = subtree
                            .as_ref()
                            .borrow()
                            .unwrap_as_subtree_unchecked()
                            .parent
                            .as_ref()
                            .map(|node| node.upgrade().unwrap())
                            .unwrap();

                        {
                            let mut parent_tree_ref = parent_tree.borrow_mut();
                            let parent_tree_ref = parent_tree_ref.unwrap_as_subtree_mut_unchecked();

                            let subtree_index = parent_tree_ref
                                .children
                                .iter()
                                .position(|node| Rc::ptr_eq(node, &subtree))
                                .unwrap();

                            parent_tree_ref.children.remove(subtree_index);
                            parent_tree_ref.children.push(first_subtree);
                            parent_tree_ref.children.push(second_subtree);
                        }

                        self.insert_mid_key_to_parent_subtree(parent_tree.clone(), mid_key);
                    }
                }
            }
        }

        #[inline]
        fn rebalance_root_after_mid_key_insertion(&mut self) {
            let (first_subtree, second_subtree, mid_key) = unsafe {
                let mut root_tree = self.root.as_ref().unwrap().borrow_mut();
                let root_tree = root_tree.unwrap_as_subtree_mut_unchecked();

                let first_subtree = Rc::new(RefCell::new(BTreeNode::SubTree {
                    subtree: BTreeSubTree::new(
                        root_tree.children[..2]
                            .iter()
                            .map(|node| node.clone())
                            .collect(),
                        None,
                        vec![root_tree.mid_keys[0].clone()],
                    ),
                }));

                root_tree.children[..2]
                    .iter_mut()
                    .for_each(|node| match &mut *node.borrow_mut() {
                        BTreeNode::Leaf { leaf } => {
                            leaf.parent = Some(Rc::downgrade(&first_subtree))
                        }

                        BTreeNode::SubTree { subtree } => {
                            subtree.parent = Some(Rc::downgrade(&first_subtree))
                        }
                    });

                let second_subtree = Rc::new(RefCell::new(BTreeNode::SubTree {
                    subtree: BTreeSubTree::new(
                        root_tree.children[2..].iter().map(|x| x.clone()).collect(),
                        None,
                        vec![root_tree.mid_keys[2].clone()],
                    ),
                }));

                root_tree.children[2..]
                    .iter_mut()
                    .for_each(|node| match &mut *node.borrow_mut() {
                        BTreeNode::Leaf { leaf } => {
                            leaf.parent = Some(Rc::downgrade(&second_subtree))
                        }

                        BTreeNode::SubTree { subtree } => {
                            subtree.parent = Some(Rc::downgrade(&second_subtree))
                        }
                    });

                (
                    first_subtree.clone(),
                    second_subtree.clone(),
                    root_tree.mid_keys[1].clone(),
                )
            };

            let new_root = Rc::new(RefCell::new(BTreeNode::SubTree {
                subtree: BTreeSubTree::new(
                    vec![first_subtree.clone(), second_subtree.clone()],
                    None,
                    vec![mid_key],
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
                    .parent = Some(Rc::downgrade(&new_root));
            }

            self.root = Some(new_root)
        }
    }

    impl<T: Ord + Eq + Clone> Extend<T> for BTree<T> {
        #[inline]
        fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
            iter.into_iter().for_each(|x| self.insert(x));
        }
    }
}

fn main() {
    let mut tree = BTree::new();
    tree.extend(0..=100);
    assert_eq!(tree.len(), 101);
}
