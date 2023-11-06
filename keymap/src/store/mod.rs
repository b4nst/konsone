use crate::ngrams::Buffer;
use log::{info, warn};
use rdev::{Event, EventType, Key};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct Store {
    pub heatmap: HashMap<Keystroke, u32>,
    pub bigram: HashMap<(Keystroke, Keystroke), u32>,
    pub trigram: HashMap<(Keystroke, Keystroke, Keystroke), u32>,

    ngrams: Buffer<EventWrapper>,
    last_save: std::time::SystemTime,
    filename: String,
}

#[derive(Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Keystroke {
    pub key: Key,
    pub interpreted: String,
}

impl Store {
    pub fn new(filename: String) -> Store {
        Store {
            heatmap: HashMap::new(),
            bigram: HashMap::new(),
            trigram: HashMap::new(),
            ngrams: Buffer::<EventWrapper>::new(),
            last_save: std::time::SystemTime::now(),
            filename: filename,
        }
    }

    pub fn process_event(&mut self, e: Event) {
        match e.event_type {
            EventType::KeyPress(_) => self.update(EventWrapper(e)),
            _ => return,
        }
    }

    fn update(&mut self, ew: EventWrapper) {
        self.ngrams.push(ew);
        let events = self.ngrams.to_vec();

        self.update_heatmap(&events[0].0);
        if events[0]
            .0
            .time
            .duration_since(events[1].0.time)
            .unwrap()
            .as_secs()
            < 2
        {
            self.update_bigram(&events[1].0, &events[0].0);
        }
        if events[0]
            .0
            .time
            .duration_since(events[2].0.time)
            .unwrap()
            .as_secs()
            < 4
        {
            self.update_trigram(&events[2].0, &events[1].0, &events[0].0);
        }

        // Store to file if last event is older than 10 minutes.
        match self.last_save.elapsed() {
            Ok(elapsed) => {
                if elapsed.as_secs() > 10 {
                    self.save().unwrap_or_else(|err| {
                        warn!("Error saving: {}", err);
                    })
                }
            }
            Err(err) => {
                warn!("Error getting elapsed time: {}", err);
                self.save().unwrap_or_else(|err| {
                    warn!("Error saving: {}", err);
                })
            }
        }
    }

    pub fn save(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Saving to {}", self.filename);

        let file = std::fs::File::create(&self.filename)?;
        serde_bare::to_writer(file, &self)?;

        self.last_save = std::time::SystemTime::now();
        Ok(())
    }

    fn update_heatmap(&mut self, e: &Event) {
        let ks = event_to_keystroke(e);
        let count = self.heatmap.entry(ks).or_insert(0);
        *count += 1;
    }

    fn update_bigram(&mut self, e1: &Event, e2: &Event) {
        let ks1 = event_to_keystroke(e1);
        let ks2 = event_to_keystroke(e2);
        let count = self.bigram.entry((ks1, ks2)).or_insert(0);
        *count += 1;
    }

    fn update_trigram(&mut self, e1: &Event, e2: &Event, e3: &Event) {
        let ks1 = event_to_keystroke(e1);
        let ks2 = event_to_keystroke(e2);
        let ks3 = event_to_keystroke(e3);
        let count = self.trigram.entry((ks1, ks2, ks3)).or_insert(0);
        *count += 1;
    }
}

pub fn load<R>(rdr: R) -> Result<Store, Box<dyn std::error::Error>>
where
    R: std::io::Read,
{
    let store = serde_bare::from_reader(rdr)?;
    Ok(store)
}

fn event_to_keystroke(e: &Event) -> Keystroke {
    Keystroke {
        key: match e.event_type {
            EventType::KeyPress(k) => k,
            EventType::KeyRelease(k) => k,
            _ => Key::Unknown(0),
        },
        interpreted: e.name.clone().unwrap_or("".to_string()),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EventWrapper(Event);

impl Default for EventWrapper {
    fn default() -> Self {
        EventWrapper(Event {
            time: std::time::UNIX_EPOCH,
            event_type: EventType::KeyPress(Key::Unknown(0)),
            name: None,
        })
    }
}