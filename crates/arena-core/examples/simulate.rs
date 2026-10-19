//! Runs seeded games with random legal actions, without recording histories.

use arena_core::{
    Game,
    games::{
        crazy_eights::CrazyEights, mancala::Mancala, shithead::Shithead, skyjo::Skyjo,
        tempel_des_schreckens::TempelDesSchreckens, the_crew::TheCrew, tic_tac_toe::TicTacToe,
    },
};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha12Rng;
use std::{env, time::Instant};

/// Measures a fixed batch of games for one ruleset.
fn simulate<G: Game>(count: usize, players: usize) -> Result<(), Box<dyn std::error::Error>> {
    let mut rng = ChaCha12Rng::from_seed([1; 32]);
    let start = Instant::now();
    let mut actions = 0;
    for _ in 0..count {
        let mut game = G::new(players, rng.r#gen())?;
        let mut steps = 0;
        while game.outcome().is_none() {
            let Some(player) = game.turn().players.first().copied() else {
                let resolution = game
                    .resolutions()
                    .first()
                    .copied()
                    .ok_or("Simulation stalled")?;
                game.resolve(resolution);
                continue;
            };
            let legal = game.legal_actions(player);
            if legal.is_empty() || steps >= 100_000 {
                return Err("Simulation stalled".into());
            }
            game.advance(player, &legal[rng.gen_range(0..legal.len())]);
            steps += 1;
        }
        actions += steps;
    }
    println!("{count} games, {actions} actions, {:?}", start.elapsed());
    Ok(())
}

/// Selects a ruleset and optional game count from command-line arguments.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let kind = args.next().unwrap_or_else(|| "mancala".into());
    let count = args.next().map(|n| n.parse()).transpose()?.unwrap_or(1000);
    match kind.as_str() {
        "mancala" => simulate::<Mancala>(count, 2),
        "crazy_eights" => simulate::<CrazyEights>(count, 2),
        "shithead" => simulate::<Shithead>(count, 2),
        "skyjo" => simulate::<Skyjo>(count, 2),
        "tic_tac_toe" => simulate::<TicTacToe>(count, 2),
        "the_crew" => simulate::<TheCrew>(count, 3),
        "tempel_des_schreckens" => simulate::<TempelDesSchreckens>(count, 3),
        _ => Err("Use mancala, crazy_eights, shithead, skyjo, tic_tac_toe, the_crew, or tempel_des_schreckens".into()),
    }
}
