use std::hash::Hash;
use std::{cell::RefCell, collections::HashMap};

use slint::{Model, ModelNotify, ModelTracker};

/// A [`Model`] backed by `HashMap<K, V>`, using interior mutability.
pub struct HashMapModel<K, V> {
    inner: RefCell<HashMap<K, V>>,
    keys: RefCell<Vec<K>>,
    notify: ModelNotify,
}

// TODO: entry()やand_modify()など？
impl<K: Eq + Hash + Clone, V: Clone> HashMapModel<K, V> {
    pub fn new() -> Self {
        Self {
            inner: Default::default(),
            keys: Default::default(),
            notify: Default::default(),
        }
    }

    /// Same as [`HashMap::insert()`].
    pub fn insert(&self, key: K, value: V) {
        if let Some(pos) = self.keys.borrow().iter().position(|k| k == &key) {
            // replace existing
            self.inner.borrow_mut().insert(key.clone(), value);
            self.notify.row_changed(pos);
        } else {
            let idx = self.keys.borrow().len();
            self.keys.borrow_mut().push(key.clone());
            self.inner.borrow_mut().insert(key, value);
            self.notify.row_added(idx, 1);
        }
    }

    /// Same as [`HashMap::remove()`].
    pub fn remove(&self, key: &K) -> Option<V> {
        if let Some(pos) = self.keys.borrow().iter().position(|k| k == key) {
            self.keys.borrow_mut().remove(pos); // O(n) shift
            let v = self.inner.borrow_mut().remove(key);
            self.notify.row_removed(pos, 1);
            v
        } else {
            None
        }
    }

    /// Similar to [`HashMap::get()`], but returns owned value due to [`RefCell`]'s borrow is temporary.
    pub fn get(&self, key: &K) -> Option<V> {
        self.inner.borrow().get(key).cloned()
    }
}

impl<K: Eq + Hash + Clone, V: Clone> Model for HashMapModel<K, V> {
    type Data = V;

    fn row_count(&self) -> usize {
        self.keys.borrow().len()
    }

    fn row_data(&self, row: usize) -> Option<Self::Data> {
        self.keys
            .borrow()
            .get(row)
            .and_then(|k| self.inner.borrow().get(k).cloned())
    }

    fn model_tracker(&self) -> &dyn ModelTracker {
        &self.notify
    }
}
