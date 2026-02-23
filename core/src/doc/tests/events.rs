use crate::doc::DocEffect;

// Utility: find index of the first event matching predicate at or after `start_at`
#[allow(dead_code)]
fn find_event_idx<F>(events: &[DocEffect], start_at: usize, pred: F) -> Option<usize>
where
    F: Fn(&DocEffect) -> bool,
{
    events
        .iter()
        .enumerate()
        .skip(start_at)
        .find_map(|(i, e)| if pred(e) { Some(i) } else { None })
}

#[test]
#[ignore = "FixtureDefのCRUDメソッド削除等に未対応"]
fn doc_handle_events_sequence_contains_expected_order() {
    /*let (handle, _doc_store, observer) = make_doc_handle_with_observer();

    // 1) Insert FixtureDef
    let def = make_fixture_def_with_mode(
        "ModelX",
        "ModeA",
        "Dimmer",
        0,
        MergeMode::LTP,
        ChannelKind::Dimmer,
    );
    let def_id = def.id();
    handle.insert_fixture_def(def);

    // 2) Add Function
    let func = make_function("Func1");
    let func_id = func.id();
    handle.add_function(func);

    // 3) Add Universe
    let uni_id = UniverseId::new(1);
    handle.add_universe(uni_id);

    // 4) Add Fixture
    let fxt = make_fixture("Fx1", def_id, uni_id, DmxAddress::new(1).unwrap(), "ModeA");
    let fxt_id = fxt.id();
    handle.add_fixture(fxt).expect("should work");

    // 5) Add Output (emits UniverseSettingsChanged)
    let plugin_id = OutputPluginId::new();
    handle.add_output(uni_id, plugin_id).unwrap();

    // 6) Remove Fixture
    handle
        .remove_fixture(&fxt_id)
        .expect("fixture removal should succeed");

    // 7) Remove FixtureDef
    handle.remove_fixture_def(&def_id);

    // 8) Remove Universe
    handle.remove_universe(&uni_id);

    let events = observer.read().unwrap().events.clone();

    // Verify subsequence presence (order matters, adjacency does not)
    let mut cur = 0usize;

    cur = find_event_idx(
        &events,
        cur,
        |e| matches!(e, DocEvent::FixtureDefAdded(id) if *id == def_id),
    )
    .expect("FixtureDefAdded not found")
        + 1;

    cur = find_event_idx(
        &events,
        cur,
        |e| matches!(e, DocEvent::FunctionAdded(id) if *id == func_id),
    )
    .expect("FunctionAdded not found")
        + 1;

    cur = find_event_idx(
        &events,
        cur,
        |e| matches!(e, DocEvent::UniverseAdded(id) if *id == uni_id),
    )
    .expect("UniverseAdded not found")
        + 1;

    cur = find_event_idx(
        &events,
        cur,
        |e| matches!(e, DocEvent::FixtureAdded(id) if *id == fxt_id),
    )
    .expect("FixtureAdded not found")
        + 1;

    cur = find_event_idx(&events, cur, |e| {
        matches!(e, DocEvent::UniverseSettingsChanged)
    })
    .expect("UniverseSettingsChanged not found")
        + 1;

    cur = find_event_idx(
        &events,
        cur,
        |e| matches!(e, DocEvent::FixtureRemoved(id) if *id == fxt_id),
    )
    .expect("FixtureRemoved not found")
        + 1;

    cur = find_event_idx(
        &events,
        cur,
        |e| matches!(e, DocEvent::FixtureDefRemoved(id) if *id == def_id),
    )
    .expect("FixtureDefRemoved not found")
        + 1;

    let _ = find_event_idx(
        &events,
        cur,
        |e| matches!(e, DocEvent::UniverseRemoved(id) if *id == uni_id),
    )
    .expect("UniverseRemoved not found");*/
}

#[test]
#[ignore = "FixtureDefのCRUDメソッド削除等に未対応"]
fn doc_handle_notifies_observer_after_lock_released() {
    todo!()
    // This test verifies that observers can safely read from DocStore
    // during on_doc_event callback without causing deadlock.
    // The key behavior is that DocHandle releases the write lock before notifying.

    /*use std::sync::{Arc, RwLock};

    use crate::doc::{DocEventBus, DocHandle, DocObserver, DocStore};

    struct ReadingObserver {
        doc_store: Arc<RwLock<DocStore>>,
        read_succeeded: bool,
    }

    impl DocObserver for ReadingObserver {
        fn on_doc_event(&mut self, _event: &DocEvent) {
            // Try to acquire a read lock - this should succeed if the write lock is released
            if let Ok(_guard) = self.doc_store.try_read() {
                self.read_succeeded = true;
            }
        }
    }

    let doc_store = Arc::new(RwLock::new(DocStore::new()));
    let mut event_bus = DocEventBus::new();

    let observer = Arc::new(RwLock::new(ReadingObserver {
        doc_store: Arc::clone(&doc_store),
        read_succeeded: false,
    }));
    let obs: Arc<RwLock<dyn DocObserver>> = Arc::clone(&observer) as _;
    event_bus.subscribe(Arc::downgrade(&obs));

    let handle = DocHandle::new(Arc::clone(&doc_store), Rc::new(RefCell::new(event_bus)));

    // Trigger an event
    let uni_id = UniverseId::new(1);
    handle.add_universe(uni_id);

    // Verify that the observer was able to read from DocStore during the callback
    assert!(
        observer.read().unwrap().read_succeeded,
        "Observer should be able to read from DocStore during on_doc_event"
    );*/
}
