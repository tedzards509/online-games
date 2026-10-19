//! Cross-ruleset checks for decks, legal actions, private views, and replays.

use arena_core::{
    Action, GameKind, Match,
    cards::{Deck, standard_deck},
};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha12Rng;

#[test]
fn deck_shuffle_draw_and_recycle() {
    let mut a = standard_deck();
    let mut b = a.clone();
    a.shuffle(&mut ChaCha12Rng::from_seed([1; 32]));
    b.shuffle(&mut ChaCha12Rng::from_seed([1; 32]));
    assert_eq!(a, b);
    let mut cards = vec![];
    while let Some(card) = a.draw() {
        cards.push(card);
    }
    cards.sort_by_key(|c| (c.rank, format!("{:?}", c.suit)));
    cards.dedup();
    assert_eq!(cards.len(), 52);
    let mut deck = Deck::new(vec![]);
    let mut discard = vec![1, 2, 3];
    deck.recycle(&mut discard, &mut ChaCha12Rng::from_seed([0; 32]));
    assert_eq!(discard, vec![3]);
    assert_eq!(deck.len(), 2);
}

#[test]
fn rejected_moves_leave_state_and_log_untouched() {
    let mut game = Match::new(GameKind::Mancala, 2, [0; 32]).unwrap();
    let before = serde_json::to_value(&game).unwrap();
    assert!(
        game.act(
            1,
            Action::Mancala(arena_core::games::mancala::Action { pit: 7 })
        )
        .is_err()
    );
    assert!(
        game.act(
            0,
            Action::Mancala(arena_core::games::mancala::Action { pit: usize::MAX })
        )
        .is_err()
    );
    assert_eq!(before, serde_json::to_value(&game).unwrap());
}

#[test]
fn all_games_finish_and_replay_exactly() {
    for kind in [
        GameKind::Mancala,
        GameKind::CrazyEights,
        GameKind::Shithead,
        GameKind::Skyjo,
        GameKind::TicTacToe,
    ] {
        for seed in 0..8 {
            let mut game = Match::new(kind, 2, [seed; 32]).unwrap();
            let mut rng = ChaCha12Rng::from_seed([seed + 8; 32]);
            for _ in 0..20000 {
                if game.outcome().is_some() {
                    break;
                }
                let player = game.turn().players[0];
                let actions = game.legal_actions(player);
                assert!(!actions.is_empty(), "{kind:?} stalled");
                let action = actions[rng.gen_range(0..actions.len())].clone();
                game.act(player, action).unwrap();
                let view = serde_json::to_value(game.view(player)).unwrap();
                match kind {
                    GameKind::Mancala => assert_eq!(
                        view["state"]["board"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .map(|v| v.as_u64().unwrap())
                            .sum::<u64>(),
                        48
                    ),
                    GameKind::CrazyEights => {
                        let counts: u64 = view["state"]["counts"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .map(|v| v.as_u64().unwrap())
                            .sum();
                        assert!(counts + view["state"]["stock"].as_u64().unwrap() < 52);
                    }
                    _ => {}
                }
            }
            assert!(
                game.outcome().is_some(),
                "{kind:?} seed {seed} did not finish"
            );
            let replay = game.replay();
            let restored = replay.simulate().unwrap();
            assert_eq!(
                serde_json::to_value(&game).unwrap(),
                serde_json::to_value(restored).unwrap()
            );
            let before = serde_json::to_value(&game).unwrap();
            assert!(
                game.act(game.turn().players[0], replay.records[0].action.clone())
                    .is_err()
            );
            assert_eq!(serde_json::to_value(game).unwrap(), before);
            let mut altered = replay.clone();
            altered.records[0].events.clear();
            assert!(altered.simulate().is_err());
        }
    }
}

#[test]
fn every_supported_player_count_completes_and_invalid_counts_are_rejected() {
    for kind in [
        GameKind::Mancala,
        GameKind::CrazyEights,
        GameKind::Shithead,
        GameKind::Skyjo,
        GameKind::TicTacToe,
    ] {
        for count in [0, 1, kind.max_players() + 1, usize::MAX] {
            assert!(Match::new(kind, count, [0; 32]).is_err());
        }
        for count in 2..=kind.max_players() {
            let mut game = Match::new(kind, count, [count as u8; 32]).unwrap();
            let mut rng = ChaCha12Rng::from_seed([count as u8; 32]);
            for _ in 0..20000 {
                if game.outcome().is_some() {
                    break;
                }
                let player = game.turn().players[0];
                let actions = game.legal_actions(player);
                assert!(!actions.is_empty(), "{kind:?} with {count} players stalled");
                game.act(player, actions[rng.gen_range(0..actions.len())].clone())
                    .unwrap();
            }
            assert!(
                game.outcome().is_some(),
                "{kind:?} with {count} players did not finish"
            );
        }
    }
}

#[test]
fn player_views_do_not_contain_private_state() {
    for kind in [GameKind::CrazyEights, GameKind::Shithead, GameKind::Skyjo] {
        let game = Match::new(kind, 2, [1; 32]).unwrap();
        let view = serde_json::to_value(game.view(0)).unwrap();
        assert!(view["state"].get("seed").is_none());
        assert!(view["state"].get("deck").is_none());
        assert!(game.legal_actions(1).is_empty());
        if kind == GameKind::Skyjo {
            assert!(
                view["state"]["grids"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .flat_map(|g| g.as_array().unwrap())
                    .all(|c| c["value"].is_null())
            );
        } else {
            let other = serde_json::to_value(game.view(1)).unwrap();
            assert_ne!(view["state"]["hand"], other["state"]["hand"]);
            assert!(view["state"].get("hands").is_none());
        }
    }
}
