use std::fs::File;
use std::io::Write;

use clap::{Args, Parser, Subcommand};
use log::{error, info, warn};
use rdev::listen;

use konsone::corpus::Generator;
use konsone::store::{load, Store};

#[derive(Parser)]
#[command(author, about, version)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Log(Log),
    Gen(Gen),
}

#[derive(Args)]
struct Log {
    filename: Option<String>,
}

#[derive(Args)]
struct Gen {
    filename: Option<String>,
}

fn main() {
    env_logger::init();

    let cli = Cli::parse();
    match cli.command {
        Commands::Log(l) => log(l.filename.unwrap_or("keymap".to_string())),
        Commands::Gen(g) => generate(g.filename.unwrap_or("keymap".to_string())),
    }
}

fn log(filename: String) {
    let mut store = match File::open(&filename) {
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

fn generate(filename: String) {
    let store = load(File::open(&filename).expect("unable to open db")).expect("unable to load db");
    let mut outf = File::create("corpus.dat").expect("creation failed");

    let corpus = Generator::new(&store.heatmap, &store.bigram, &store.trigram);

    for ks in corpus {
        outf.write(ks.interpreted.as_bytes()).expect("write failed");
    }
}
