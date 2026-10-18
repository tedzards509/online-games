//! Loads an exported replay, validates its history, and prints the final result.

use arena_core::Replay;

/// Reads the replay path from the first command-line argument.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).ok_or("Usage: replay <file.json>")?;
    let replay: Replay = serde_json::from_reader(std::fs::File::open(path)?)?;
    let game = replay.simulate()?;
    println!(
        "{} actions; outcome: {:?}",
        replay.records.len(),
        game.outcome()
    );
    Ok(())
}
