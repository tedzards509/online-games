use arena_core::Replay;

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
