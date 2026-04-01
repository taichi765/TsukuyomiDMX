#![allow(unused)]
use std::time::Duration;

pub struct TimeLine {
    id: usize,
    name: String,
    tracks: Vec<Track>,
}

impl TimeLine {
    fn new(id: usize, name: &str) -> Self {
        Self {
            id,
            name: name.into(),
            tracks: Vec::new(),
        }
    }
}

struct Track {
    index: usize,
    name: String,
    items: Vec<TrackItem>,
}

struct TrackItem {
    function_id: usize,
    start_time: Duration,
}
