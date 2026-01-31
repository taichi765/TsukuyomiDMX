use i_slint_core::model::{ModelChangeListener, ModelChangeListenerContainer};
use slint::{Model, ModelNotify};
use std::{
    any::Any, cell::RefCell, collections::HashMap, fmt::Debug, hash::Hash, pin::Pin, rc::Rc,
};

/// Similar to Kotlin's `Flow.runningFold()`.
///
/// closure's first arg is immutable reference, so you need to use interior mutability in `T`.
pub struct ScanModel<Ev, T, F>(Pin<Box<ModelChangeListenerContainer<ScanModelInner<Ev, T, F>>>>)
where
    Ev: Clone + Debug + 'static,
    T: Model + Default + 'static,
    F: Fn(&T, Ev) + 'static;

impl<Ev, T, F> ScanModel<Ev, T, F>
where
    Ev: Clone + Debug + 'static,
    T: Model + Default + 'static,
    F: Fn(&T, Ev) + 'static,
{
    pub fn new(source: Rc<EventModel<Ev>>, init: T, f: F) -> Self {
        let inner = Box::pin(ModelChangeListenerContainer::new(ScanModelInner {
            source: Rc::clone(&source),
            state: RefCell::new(init),
            f,
            notify: Default::default(),
        }));
        source
            .model_tracker()
            .attach_peer(inner.as_ref().model_peer());
        Self(inner)
    }
}

impl<Ev, T, U, F> Model for ScanModel<Ev, T, F>
where
    Ev: Clone + Debug + 'static,
    T: Model<Data = U> + Default + 'static,
    U: 'static,
    F: Fn(&T, Ev) + 'static,
{
    type Data = U;

    fn row_count(&self) -> usize {
        self.0.state.borrow().row_count()
    }

    fn row_data(&self, row: usize) -> Option<Self::Data> {
        self.0.state.borrow().row_data(row)
    }

    fn model_tracker(&self) -> &dyn slint::ModelTracker {
        &self.0.notify
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

struct ScanModelInner<Ev, T, F> {
    source: Rc<EventModel<Ev>>,
    state: RefCell<T>,
    f: F,
    notify: ModelNotify,
}

impl<Ev, T, F> ModelChangeListener for ScanModelInner<Ev, T, F>
where
    Ev: Clone + Debug + 'static,
    T: Model + Default + 'static,
    F: Fn(&T, Ev) + 'static,
{
    fn row_added(self: Pin<&Self>, index: usize, count: usize) {
        debug_assert_eq!(1, count);
        (self.f)(&self.state.borrow(), self.source.get(index).unwrap());
    }

    fn row_changed(self: Pin<&Self>, _row: usize) {
        debug_assert!(false, "EventModel is append-only")
    }

    fn row_removed(self: Pin<&Self>, _index: usize, _count: usize) {
        debug_assert!(false, "EventModel is append-only")
    }

    fn reset(self: Pin<&Self>) {
        debug_assert!(false, "EventModel is append-only")
    }
}

/// Append-only model.
#[derive(derive_more::Debug)]
pub struct EventModel<T> {
    inner: RefCell<Vec<T>>,
    #[debug(skip)]
    notify: ModelNotify,
}

impl<T> EventModel<T>
where
    T: Clone + 'static,
{
    pub fn new() -> Self {
        Self {
            inner: RefCell::new(Vec::new()),
            notify: Default::default(),
        }
    }

    pub fn append(&self, val: T) {
        self.inner.borrow_mut().push(val);
        self.notify.row_added(self.inner.borrow().len() - 1, 1);
    }

    pub fn get(&self, index: usize) -> Option<T> {
        self.inner.borrow().get(index).cloned() // TODO: Option<&T>にしたい
    }
}

impl<T> Model for EventModel<T>
where
    T: Clone + 'static,
{
    type Data = T;

    fn row_count(&self) -> usize {
        self.inner.borrow().len()
    }

    fn row_data(&self, row: usize) -> Option<Self::Data> {
        if row < self.inner.borrow().len() {
            Some(self.inner.borrow()[row].clone())
        } else {
            None
        }
    }

    fn model_tracker(&self) -> &dyn slint::ModelTracker {
        &self.notify
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

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

    fn model_tracker(&self) -> &dyn slint::ModelTracker {
        &self.notify
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use slint::VecModel;

    #[derive(Clone, Debug)]
    enum TestEvent {
        Added(u32),
        Removed(u32),
    }

    #[test]
    fn fold_model_works() {
        let events: Rc<EventModel<TestEvent>> = Rc::new(EventModel::new());
        let model = ScanModel::new(
            Rc::clone(&events),
            VecModel::from(Vec::new()),
            |state, ev| match ev {
                TestEvent::Added(id) => {
                    state.push(id);
                }
                TestEvent::Removed(id) => {
                    let pos = state.iter().position(|v| v == id).unwrap();
                    state.remove(pos);
                }
            },
        );

        let added_id = 2;
        events.append(TestEvent::Added(added_id));
        matches!(model.0.state.borrow().row_data(0), Some(id) if id == added_id);
        assert_eq!(model.0.state.borrow().row_count(), 1);

        let added_id2 = 6;
        events.append(TestEvent::Added(added_id2));
        matches!(model.0.state.borrow().row_data(1), Some(id) if id == added_id2);
        assert_eq!(model.0.state.borrow().row_count(), 2);

        events.append(TestEvent::Removed(added_id));
        matches!(model.0.state.borrow().row_data(0), Some(id) if id == added_id2);
        assert_eq!(model.0.state.borrow().row_count(), 1);
    }
}
