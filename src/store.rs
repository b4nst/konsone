use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use log::{info, warn};
use rdev::{Event, EventType, Key};
use serde::{Deserialize, Serialize};

use crate::corpus::{BigramHeatmap, Keystroke, KeystrokeHeatmap, TrigramHeatmap};
use crate::tribuf::Buffer;

const MAX_KEY_DELAY: u64 = 2;

/// Store for keypresses.
#[derive(Serialize, Deserialize)]
pub struct Store {
    /// Heatmap of keypresses.
    pub heatmap: KeystrokeHeatmap,
    /// Heatmap of bigrams.
    pub bigram: BigramHeatmap,
    /// Heatmap of trigrams.
    pub trigram: TrigramHeatmap,

    ngrams: Buffer<EventWrapper>,
    last_save: std::time::SystemTime,
    filename: String,
}

impl Store {
    /// Create a new store.
    pub fn new(filename: String) -> Store {
        Store {
            heatmap: HashMap::new(),
            bigram: HashMap::new(),
            trigram: HashMap::new(),
            ngrams: Buffer::<EventWrapper>::new(),
            last_save: SystemTime::now(),
            filename: filename,
        }
    }

    /// Process a device event.
    pub fn process_event(&mut self, e: Event) {
        match e.event_type {
            EventType::KeyPress(_) => self.update(EventWrapper(e)),
            _ => return,
        }
    }

    /// Save the store to the filesystem.
    pub fn save(&mut self) -> Result<(), Box<dyn Error>> {
        info!("Saving to {}", self.filename);

        let file = File::create(&self.filename)?;
        serde_bare::to_writer(file, &self)?;

        self.last_save = std::time::SystemTime::now();
        Ok(())
    }

    fn update(&mut self, ew: EventWrapper) {
        self.ngrams.push(ew);
        let events: Vec<Event> = self.ngrams.to_vec().into_iter().map(|ew| ew.0).collect();

        self.update_heatmap(&events[0]);
        if is_within_delay(&events[0], &events[1]) {
            self.update_bigram(&events[1], &events[0]);
            if is_within_delay(&events[1], &events[2]) {
                self.update_trigram(&events[2], &events[1], &events[0]);
            }
        }

        // Store to file if last event is older than 10 minutes.
        match self.last_save.elapsed() {
            Ok(elapsed) => {
                if elapsed.as_secs() > 600 {
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

/// Load a store from a reader.
pub fn load<R>(rdr: R) -> Result<Store, Box<dyn Error>>
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

/// Check if 2 events are occuring within the maximum key delay.
fn is_within_delay(e1: &Event, e2: &Event) -> bool {
    e1.time
        .duration_since(e2.time)
        .unwrap_or_else(|err| {
            warn!("Error getting elapsed time between events: {}", err);
            Duration::from_secs(MAX_KEY_DELAY + 1)
        })
        .as_secs()
        < MAX_KEY_DELAY
}

/// Wrapper for an event to allow for Default trait implementation.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct EventWrapper(Event);

impl Default for EventWrapper {
    fn default() -> Self {
        EventWrapper(Event {
            time: UNIX_EPOCH,
            event_type: EventType::KeyPress(Key::Unknown(0)),
            name: None,
        })
    }
}

// Tests

#[cfg(test)]
mod tests {
    use super::*;
    use std::ops::Add;

    #[test]
    fn test_event_to_keystroke() {
        let e = Event {
            time: UNIX_EPOCH,
            event_type: EventType::KeyPress(Key::KeyA),
            name: None,
        };
        let ks = event_to_keystroke(&e);
        assert_eq!(
            ks,
            Keystroke {
                key: Key::KeyA,
                interpreted: "".to_string(),
            }
        );
    }

    #[test]
    fn test_store_process_events() {
        let mut store = Store::new("".to_string());
        let ka = Keystroke {
            key: Key::KeyA,
            interpreted: "a".to_string(),
        };
        let kb = Keystroke {
            key: Key::KeyB,
            interpreted: "b".to_string(),
        };

        store.process_event(Event {
            time: SystemTime::now(),
            event_type: EventType::KeyPress(Key::KeyA),
            name: "a".to_string().into(),
        });
        store.process_event(Event {
            time: SystemTime::now(),
            event_type: EventType::KeyPress(Key::KeyA),
            name: "a".to_string().into(),
        });
        store.process_event(Event {
            time: SystemTime::now(),
            event_type: EventType::KeyPress(Key::KeyB),
            name: "b".to_string().into(),
        });
        // This event is too far in the future to be considered a bigram or trigram.
        store.process_event(Event {
            time: SystemTime::now().add(Duration::from_secs(3)),
            event_type: EventType::KeyPress(Key::KeyB),
            name: "b".to_string().into(),
        });
        assert_eq!(
            store.heatmap.len(),
            2,
            "Expected 2 keystrokes, got {:?}",
            store.heatmap
        );
        assert_eq!(
            store.bigram.len(),
            2,
            "Expected 2 bigrams, got {:?}",
            store.bigram
        );
        assert_eq!(
            store.trigram.len(),
            1,
            "Expected 1 trigram, got {:?}",
            store.trigram
        );
        assert_eq!(store.heatmap[&ka], 2);
        assert_eq!(store.heatmap[&kb], 2);
        assert_eq!(store.bigram[&(ka.clone(), ka.clone())], 1);
        assert_eq!(store.bigram[&(ka.clone(), kb.clone())], 1);
        assert_eq!(store.trigram[&(ka.clone(), ka.clone(), kb.clone())], 1);
    }
}
