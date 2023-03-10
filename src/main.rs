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

    #[derive(Debug, Clone)]
    enum BTreeNode<T: Ord + Eq + Clone> {
        Leaf { leaf: BTreeLeaf<T> },
        SubTree { subtree: BTreeSubTree<T> },
    }

    #[derive(Debug, Default, Clone)]
    struct BTreeLeaf<T: Ord + Eq + Clone> {
        values: Vec<Rc<T>>,
        parent: Option<Weak<RefCell<BTreeNode<T>>>>,
        next_leaf: Option<Rc<RefCell<BTreeNode<T>>>>,
        previous_leaf: Option<Weak<RefCell<BTreeNode<T>>>>,
    }

    #[derive(Debug, Clone)]
    pub struct BTreeIter<T: Ord + Eq + Clone> {
        cur_leaf: Option<Rc<RefCell<BTreeNode<T>>>>,
        cur_ind: usize,
    }

    #[derive(Debug, Default, Clone)]
    struct BTreeSubTree<T: Ord + Eq + Clone> {
        children: Vec<Rc<RefCell<BTreeNode<T>>>>,
        parent: Option<Weak<RefCell<BTreeNode<T>>>>,
        mid_keys: Vec<Rc<T>>,
        values_number: usize,
    }

    #[derive(Debug, Default, Clone)]
    pub struct BTree<T: Ord + Eq + Clone> {
        root: Option<Rc<RefCell<BTreeNode<T>>>>,
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

    impl<T: Ord + Eq + Clone> BTreeIter<T> {
        #[inline]
        fn new(cur_leaf: Option<Rc<RefCell<BTreeNode<T>>>>, cur_ind: usize) -> Self {
            Self { cur_leaf, cur_ind }
        }
    }

    impl<T: Ord + Eq + Clone> Default for BTreeIter<T> {
        #[inline]
        fn default() -> Self {
            Self {
                cur_leaf: None,
                cur_ind: 0,
            }
        }
    }

    impl<T: Ord + Eq + Clone> Iterator for BTreeIter<T> {
        type Item = Rc<T>;

        #[inline]
        fn next(&mut self) -> Option<Self::Item> {
            self.cur_leaf
                .as_ref()
                .map(|leaf| unsafe {
                    let leaf = leaf.borrow();
                    let leaf = leaf.unwrap_as_leaf_unchecked();
                    let len = leaf.values.len();

                    if self.cur_ind + 1 < len {
                        Err(())
                    } else {
                        Ok(leaf.next_leaf.as_ref().map(|next_leaf| next_leaf.clone()))
                    }
                })
                .map(|next| {
                    let output_index = self.cur_ind;
                    let cur_val =
                        self.cur_leaf.as_ref().unwrap().borrow().get_values()[output_index].clone();

                    match next {
                        Err(_) => self.cur_ind += 1,

                        Ok(next_leaf) => {
                            self.cur_ind = 0;
                            self.cur_leaf = next_leaf
                        }
                    }

                    cur_val
                })
        }
    }

    impl<T: Ord + Eq + Clone> DoubleEndedIterator for BTreeIter<T> {
        #[inline]
        fn next_back(&mut self) -> Option<Self::Item> {
            self.cur_leaf
                .as_ref()
                .map(|leaf| unsafe {
                    let leaf = leaf.borrow();
                    let leaf = leaf.unwrap_as_leaf_unchecked();

                    if self.cur_ind > 0 {
                        Err(())
                    } else {
                        Ok(leaf
                            .previous_leaf
                            .as_ref()
                            .map(|prev_leaf| prev_leaf.upgrade().map(|prev_leaf| prev_leaf.clone()))
                            .flatten())
                    }
                })
                .map(|prev| {
                    let output_index = self.cur_ind;
                    let cur_val =
                        self.cur_leaf.as_ref().unwrap().borrow().get_values()[output_index].clone();

                    match prev {
                        Err(_) => self.cur_ind -= 1,

                        Ok(prev_leaf) => {
                            self.cur_ind = prev_leaf
                                .as_ref()
                                .map(|leaf| unsafe {
                                    leaf.borrow().unwrap_as_leaf_unchecked().values.len()
                                })
                                .unwrap_or_default();

                            self.cur_leaf = prev_leaf
                        }
                    }

                    cur_val
                })
        }
    }

    impl<T: Ord + Eq + Clone> BTreeSubTree<T> {
        #[inline]
        pub fn new(
            children: Vec<Rc<RefCell<BTreeNode<T>>>>,
            parent: Option<Weak<RefCell<BTreeNode<T>>>>,
            mid_keys: Vec<Rc<T>>,
        ) -> Self {
            let values_number = children
                .iter()
                .map(|node| BTreeNode::values_number(node.clone()))
                .sum();

            Self {
                children,
                parent,
                mid_keys,
                values_number,
            }
        }

        #[inline]
        pub fn get_children_index_by_value(&self, value: &T) -> usize {
            match self.mid_keys.len() {
                1 => match value.cmp(&*self.mid_keys[0]) {
                    Ordering::Less => 0,
                    _ => 1,
                },

                2 => {
                    if *value < *self.mid_keys[0] {
                        0
                    } else if *value > *self.mid_keys[0] && *value < *self.mid_keys[1] {
                        1
                    } else {
                        2
                    }
                }

                _ => unreachable!(),
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

        #[inline]
        pub fn get_parent(&self) -> Option<&Weak<RefCell<BTreeNode<T>>>> {
            match self {
                BTreeNode::Leaf { leaf } => leaf.parent.as_ref(),
                BTreeNode::SubTree { subtree } => subtree.parent.as_ref(),
            }
        }

        #[inline]
        pub fn get_parent_mut(&mut self) -> Option<&mut Weak<RefCell<BTreeNode<T>>>> {
            match self {
                BTreeNode::Leaf { leaf } => leaf.parent.as_mut(),
                BTreeNode::SubTree { subtree } => subtree.parent.as_mut(),
            }
        }

        #[inline]
        pub fn set_parent(&mut self, new_parent: Option<Weak<RefCell<BTreeNode<T>>>>) {
            match self {
                BTreeNode::Leaf { leaf } => leaf.parent = new_parent,
                BTreeNode::SubTree { subtree } => subtree.parent = new_parent,
            }
        }

        #[inline]
        pub fn get_values(&self) -> &Vec<Rc<T>> {
            match self {
                BTreeNode::Leaf { leaf } => &leaf.values,
                BTreeNode::SubTree { subtree } => &subtree.mid_keys,
            }
        }

        #[inline]
        pub fn get_values_mut(&mut self) -> &mut Vec<Rc<T>> {
            match self {
                BTreeNode::Leaf { leaf } => &mut leaf.values,
                BTreeNode::SubTree { subtree } => &mut subtree.mid_keys,
            }
        }

        pub fn first_leaf(this: Rc<RefCell<Self>>) -> Rc<RefCell<Self>> {
            match {
                let is_leaf = this.borrow().is_leaf();
                is_leaf
            } {
                true => this,

                false => BTreeNode::first_leaf(unsafe {
                    this.borrow()
                        .unwrap_as_subtree_unchecked()
                        .children
                        .first()
                        .unwrap()
                        .clone()
                }),
            }
        }

        #[inline]
        pub fn first(this: Rc<RefCell<Self>>) -> Option<Rc<T>> {
            unsafe {
                Self::first_leaf(this)
                    .borrow()
                    .unwrap_as_leaf_unchecked()
                    .values
                    .first()
                    .map(|v| v.clone())
            }
        }

        pub fn last_leaf(this: Rc<RefCell<Self>>) -> Rc<RefCell<Self>> {
            match {
                let is_leaf = this.borrow().is_leaf();
                is_leaf
            } {
                true => this,

                false => BTreeNode::last_leaf(unsafe {
                    this.borrow()
                        .unwrap_as_subtree_unchecked()
                        .children
                        .last()
                        .unwrap()
                        .clone()
                }),
            }
        }

        #[inline]
        pub fn last(this: Rc<RefCell<Self>>) -> Option<Rc<T>> {
            unsafe {
                Self::last_leaf(this)
                    .borrow()
                    .unwrap_as_leaf_unchecked()
                    .values
                    .last()
                    .map(|v| v.clone())
            }
        }

        #[inline]
        pub fn values_number(this: Rc<RefCell<Self>>) -> usize {
            match &*this.borrow() {
                BTreeNode::Leaf { leaf } => leaf.values.len(),
                BTreeNode::SubTree { subtree } => subtree.values_number,
            }
        }

        pub fn update_parent_value_number(parent: Rc<RefCell<Self>>) {
            unsafe {
                parent
                    .borrow_mut()
                    .unwrap_as_subtree_mut_unchecked()
                    .values_number += 1;
            }

            unsafe {
                if let Some(next_parent) = &parent.borrow().unwrap_as_subtree_unchecked().parent {
                    Self::update_parent_value_number(next_parent.upgrade().unwrap().clone())
                }
            }
        }

        pub fn get(this: Rc<RefCell<Self>>, index: usize) -> Rc<T> {
            match {
                let is_leaf = this.borrow().is_leaf();
                is_leaf
            } {
                true => unsafe { this.borrow().unwrap_as_leaf_unchecked().values[index].clone() },

                false => {
                    let mut reduced_index = index;

                    let child = unsafe {
                        let this_ref = this.borrow();

                        this_ref
                            .unwrap_as_subtree_unchecked()
                            .children
                            .iter()
                            .skip_while(|&node| {
                                let values_number = Self::values_number(node.clone());

                                if reduced_index < values_number {
                                    false
                                } else {
                                    reduced_index -= values_number;
                                    true
                                }
                            })
                            .next()
                            .unwrap()
                            .clone()
                    };

                    Self::get(child, reduced_index)
                }
            }
        }

        pub fn find(this: Rc<RefCell<Self>>, value: &T) -> Rc<RefCell<Self>> {
            match {
                let is_leaf = this.borrow().is_leaf();
                is_leaf
            } {
                true => this,

                false => unsafe {
                    let this_ref = this.borrow();
                    let this_ref = this_ref.unwrap_as_subtree_unchecked();
                    let child_index = this_ref.get_children_index_by_value(value);
                    let child = this_ref.children[child_index].clone();
                    Self::find(child, value)
                },
            }
        }
    }

    impl<T: Ord + Eq + Clone> BTree<T> {
        #[inline]
        pub const fn new() -> Self {
            Self { root: None }
        }

        #[inline]
        pub fn len(&self) -> usize {
            self.root
                .as_ref()
                .map(|node| BTreeNode::values_number(node.clone()))
                .unwrap_or_default()
        }

        #[inline]
        pub fn is_empty(&self) -> bool {
            self.len() == 0
        }

        #[inline]
        pub fn is_not_empty(&self) -> bool {
            !self.is_empty()
        }

        #[inline]
        fn new_root_after_division(
            first_node: Rc<RefCell<BTreeNode<T>>>,
            second_node: Rc<RefCell<BTreeNode<T>>>,
            mid_key: Rc<T>,
        ) -> Rc<RefCell<BTreeNode<T>>> {
            let new_root = Rc::new(RefCell::new(BTreeNode::SubTree {
                subtree: BTreeSubTree::new(
                    vec![first_node.clone(), second_node.clone()],
                    None,
                    vec![mid_key],
                ),
            }));

            first_node
                .borrow_mut()
                .set_parent(Some(Rc::downgrade(&new_root)));

            second_node
                .borrow_mut()
                .set_parent(Some(Rc::downgrade(&new_root)));

            new_root
        }

        #[inline]
        pub fn insert(&mut self, value: T) {
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

            self.root = Some(Self::new_root_after_division(
                first_leaf,
                second_leaf,
                mid_key,
            ));
        }

        #[inline]
        fn insert_to_subtree(&mut self, subtree: Rc<RefCell<BTreeNode<T>>>, value: T) {
            let child_subtree_index = unsafe {
                subtree
                    .borrow()
                    .unwrap_as_subtree_unchecked()
                    .get_children_index_by_value(&value)
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
                let subtree_ref = subtree.borrow();
                let subtree_ref = subtree_ref.unwrap_as_subtree_unchecked();
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
            let (parent_tree, first_leaf, second_leaf, mid_key) = unsafe {
                let mut leaf_ref = leaf.borrow_mut();
                let leaf_ref = leaf_ref.unwrap_as_leaf_mut_unchecked();

                leaf_ref.values.push(Rc::new(value));
                leaf_ref.values.sort();

                if leaf_ref.values.len() <= MAX_KEYS {
                    let parent_tree = leaf_ref.parent.as_ref().unwrap().upgrade().unwrap().clone();
                    BTreeNode::update_parent_value_number(parent_tree);
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
                let mid_key = leaf_ref.values[1].clone();
                (parent_tree, first_leaf, second_leaf, mid_key)
            };

            BTreeNode::update_parent_value_number(parent_tree.clone());

            unsafe {
                let mut parent_subtree = parent_tree.borrow_mut();
                let parent_subtree = parent_subtree.unwrap_as_subtree_mut_unchecked();
                parent_subtree.children.remove(leaf_ind);
                parent_subtree.children.insert(leaf_ind, first_leaf);
                parent_subtree.children.insert(leaf_ind + 1, second_leaf);
            }

            self.insert_mid_key_to_parent_subtree(parent_tree, mid_key)
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
                            node.borrow_mut()
                                .set_parent(Some(Rc::downgrade(&first_subtree)))
                        });

                        let second_subtree = Rc::new(RefCell::new(BTreeNode::SubTree {
                            subtree: BTreeSubTree::new(
                                tree.children[2..].iter().map(|x| x.clone()).collect(),
                                Some(tree.parent.as_ref().unwrap().clone()),
                                vec![tree.mid_keys[2].clone()],
                            ),
                        }));

                        tree.children[2..].iter_mut().for_each(|node| {
                            node.borrow_mut()
                                .set_parent(Some(Rc::downgrade(&second_subtree)))
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

                            parent_tree_ref
                                .children
                                .insert(subtree_index, first_subtree);

                            parent_tree_ref
                                .children
                                .insert(subtree_index + 1, second_subtree);
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

                root_tree.children[..2].iter_mut().for_each(|node| {
                    node.borrow_mut()
                        .set_parent(Some(Rc::downgrade(&first_subtree)))
                });

                let second_subtree = Rc::new(RefCell::new(BTreeNode::SubTree {
                    subtree: BTreeSubTree::new(
                        root_tree.children[2..].iter().map(|x| x.clone()).collect(),
                        None,
                        vec![root_tree.mid_keys[2].clone()],
                    ),
                }));

                root_tree.children[2..].iter_mut().for_each(|node| {
                    node.borrow_mut()
                        .set_parent(Some(Rc::downgrade(&second_subtree)))
                });

                (
                    first_subtree.clone(),
                    second_subtree.clone(),
                    root_tree.mid_keys[1].clone(),
                )
            };

            self.root = Some(Self::new_root_after_division(
                first_subtree,
                second_subtree,
                mid_key,
            ))
        }

        #[inline]
        pub fn first(&self) -> Option<Rc<T>> {
            self.root
                .as_ref()
                .map(|root_node| BTreeNode::first(root_node.clone()))
                .flatten()
        }

        #[inline]
        pub fn last(&self) -> Option<Rc<T>> {
            self.root
                .as_ref()
                .map(|root_node| BTreeNode::last(root_node.clone()))
                .flatten()
        }

        #[inline]
        pub fn iter(&self) -> BTreeIter<T> {
            self.root
                .as_ref()
                .map(|root_node| BTreeNode::first_leaf(root_node.clone()))
                .map(|first_leaf| BTreeIter::new(Some(first_leaf), 0))
                .unwrap_or_default()
        }

        #[inline]
        pub unsafe fn get_unchecked(&self, index: usize) -> Rc<T> {
            BTreeNode::get(self.root.as_ref().unwrap().clone(), index)
        }

        #[inline]
        pub fn get(&self, index: usize) -> Option<Rc<T>> {
            if index >= self.len() {
                None
            } else {
                unsafe { Some(self.get_unchecked(index)) }
            }
        }

        #[inline]
        pub fn find(&self, value: &T) -> BTreeIter<T> {
            self.root
                .as_ref()
                .map(|node| BTreeNode::find(node.clone(), value))
                .map(|leaf| {
                    let cur_ind = unsafe {
                        leaf.borrow()
                            .unwrap_as_leaf_unchecked()
                            .values
                            .iter()
                            .position(|v| **v >= *value)
                    };

                    (leaf, cur_ind)
                })
                .map(|(leaf, cur_ind)| cur_ind.map(|cur_ind| BTreeIter::new(Some(leaf), cur_ind)))
                .flatten()
                .unwrap_or_default()
        }
    }

    impl<T: Ord + Eq + Clone> Extend<T> for BTree<T> {
        #[inline]
        fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
            iter.into_iter().for_each(|x| self.insert(x));
        }
    }

    impl<T: Ord + Eq + Clone> FromIterator<T> for BTree<T> {
        #[inline]
        fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
            let mut tree = BTree::new();
            tree.extend(iter.into_iter());
            tree
        }
    }

    impl<T: Ord + Eq + Clone> IntoIterator for BTree<T> {
        type Item = Rc<T>;
        type IntoIter = BTreeIter<T>;

        #[inline]
        fn into_iter(self) -> Self::IntoIter {
            self.root
                .map(|root_node| BTreeNode::first_leaf(root_node))
                .map(|first_leaf| BTreeIter::new(Some(first_leaf), 0))
                .unwrap_or_default()
        }
    }

    #[test]
    fn tree_test() {
        let tree = BTree::from_iter(-1000..=1000);
        assert_eq!(tree.len(), 2001);
        assert_eq!(tree.first().map(|x| *x), Some(-1000));
        assert_eq!(tree.last().map(|x| *x), Some(1000));

        assert!((0..tree.len())
            .map(|i| *tree.get(i).unwrap())
            .zip(-1000..=1000)
            .all(|(tree_elem, val)| { tree_elem == val }));

        assert!(tree
            .iter()
            .map(|v| *v + *v)
            .zip((-1000..).map(|x| x + x))
            .all(|(tree_elem, x)| tree_elem == x));

        assert_eq!(
            tree.iter().map(|x| *x * *x).fold(0, |acc, x| acc + x),
            (-1000..=1000).fold(0, |acc, x| acc + x * x)
        );

        assert!(tree
            .into_iter()
            .map(|v| *v * *v)
            .zip((-1000..).map(|x| x * x))
            .all(|(tree_elem, x)| tree_elem == x));
    }
}

fn main() {}
