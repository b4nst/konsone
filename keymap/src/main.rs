mod ngrams;
mod store;

use crate::store::{load, Store};
use log::{error, info, warn};
use rdev::listen;

fn main() {
    env_logger::init();

    let filename = std::env::args().nth(1).unwrap_or("keymap".to_string());
    info!("Loading store from {}", filename);

    let mut store = match std::fs::File::open(&filename) {
        Ok(file) => load(file).unwrap_or_else(|err| {
            warn!("Error loading: {}", err);
            warn!("Creating new store");
            Store::new(filename)
        }),
        Err(_) => Store::new(filename),
    };

    info!("Listening for events");

    // This will block.
    if let Err(error) = listen(move |event| store.process_event(event)) {
        error!("Error: {:?}", error);
    }
}
