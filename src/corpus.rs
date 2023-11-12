use std::collections::HashMap;

use rand::distributions::{Distribution, WeightedIndex};
use rand::rngs::ThreadRng;
use rdev::Key;
use serde::{Deserialize, Serialize};

/// Alias to identify the occurrence of a keypress.
pub type KeystrokeCount = u32;

/// A keystroke is a keypress with an OS interpreted value.
#[derive(Debug, Eq, Hash, PartialEq, Serialize, Deserialize, Clone)]
pub struct Keystroke {
    /// The key that was pressed.
    pub key: Key,
    /// The OS interpreted value of the key.
    pub interpreted: String,
}

/// A corpus is a list of keystrokes
pub type Corpus = Vec<Keystroke>;

/// A keystroke heatmap is a map of keystrokes to the number of times they have been pressed.
pub type KeystrokeHeatmap = HashMap<Keystroke, KeystrokeCount>;
/// A bigram heatmap is a map of bigrams to the number of times they have been pressed.
pub type BigramHeatmap = HashMap<(Keystroke, Keystroke), KeystrokeCount>;
/// A trigram heatmap is a map of trigrams to the number of times they have been pressed.
pub type TrigramHeatmap = HashMap<(Keystroke, Keystroke, Keystroke), KeystrokeCount>;

/// A generator is a pseudo random Keystroke generator based on typing heatmaps.
#[derive(Clone, Debug)]
pub struct Generator {
    /// The list of all keystrokes
    keystrokes: Vec<Keystroke>,
    /// The weights of each keystroke
    weights: Vec<u32>,
    /// Lookup table for bigrams to a vector of (index, weight) possible next keystroke
    bigram_lookup: HashMap<usize, Vec<(usize, KeystrokeCount)>>,
    /// Lookup table for trigrams to a vector of (index, weight) possible next keystroke
    trigram_lookup: HashMap<(usize, usize), Vec<(usize, KeystrokeCount)>>,
    /// The last two keystrokes index
    preceeding: [Option<usize>; 2],
    /// Random number generator
    rng: ThreadRng,
}

impl Generator {
    /// Create a new generator from heatmaps.
    pub fn new(
        keystrokes: &KeystrokeHeatmap,
        bigrams: &BigramHeatmap,
        trigrams: &TrigramHeatmap,
    ) -> Generator {
        // Unzip the keystrokes and weights
        let (keys, weights): (Vec<_>, Vec<_>) = keystrokes.clone().into_iter().unzip();
        let keylookup: HashMap<Keystroke, usize> = keys
            .iter()
            .cloned()
            .enumerate()
            .map(|(i, v)| (v, i))
            .collect();

        let mut bigram_lookup = HashMap::new();
        bigrams.iter().for_each(|(k, v)| {
            let index = keylookup[&k.0];
            let bigram = bigram_lookup.entry(index).or_insert(Vec::new());
            bigram.push((keylookup[&k.1], *v));
        });

        let mut trigram_lookup = HashMap::new();
        trigrams.iter().for_each(|(k, v)| {
            let index = (keylookup[&k.0], keylookup[&k.1]);
            let trigram = trigram_lookup.entry(index).or_insert(Vec::new());
            trigram.push((keylookup[&k.2], *v));
        });

        Generator {
            keystrokes: keys,
            weights,
            bigram_lookup,
            trigram_lookup,
            preceeding: [None, None],
            rng: rand::thread_rng(),
        }
    }

    /// Generate a random keystroke
    pub fn generate_random_keystroke(&mut self) -> Keystroke {
        let mut weights = self.weights.clone();

        // Update weights with bigram if we have a preceeding keystroke
        if let Some(index) = self.preceeding[0] {
            self.bigram_lookup
                .get(&index)
                .unwrap_or(&Vec::new())
                .iter()
                .for_each(|(i, w)| weights[*i] += w);
        }
        // Update weights with trigram if we have two preceeding keystrokes
        if let Some(index) = self.preceeding[1] {
            self.trigram_lookup
                .get(&(index, self.preceeding[0].unwrap()))
                .unwrap_or(&Vec::new())
                .iter()
                .for_each(|(i, w)| weights[*i] += w);
        }
        // generate weighted index
        let weighted_index = WeightedIndex::new(&weights).expect("weights index should be valid");

        // generate the next index
        let index = weighted_index.sample(&mut self.rng);
        self.preceeding[1] = self.preceeding[0];
        self.preceeding[0] = Some(index);
        // return the keystroke
        self.keystrokes[index].clone()
    }
}

impl Iterator for Generator {
    type Item = Keystroke;

    fn next(&mut self) -> Option<Self::Item> {
        let keystroke = self.generate_random_keystroke();
        Some(keystroke)
    }
}
