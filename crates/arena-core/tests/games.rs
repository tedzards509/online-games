//! Cross-ruleset checks for decks, legal actions, private views, and replays.

use arena_core::{
    Action, GameKind, Match,
    cards::{Deck, standard_deck},
};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha12Rng;

#[test]
fn crew_full_deals_conserve_cards_persist_and_replay_until_host_decides() {
    use arena_core::{View, games::the_crew::Action as CrewAction};
    for players in 3..=5 {
        for seed in 0..8 {
            let mut game = Match::new(GameKind::TheCrew, players, [seed; 32]).unwrap();
            let mut rng = ChaCha12Rng::from_seed([seed + 8; 32]);
            let mut played = 0;
            loop {
                let View::TheCrew(view) = game.view(0) else {
                    unreachable!()
                };
                assert_eq!(view.counts.iter().sum::<usize>() + played, 40);
                if view.awaiting_result {
                    break;
                }
                let seat = view.next_player;
                let plays: Vec<_> = game
                    .legal_actions(seat)
                    .into_iter()
                    .filter(|a| matches!(a, Action::TheCrew(CrewAction::Play { .. })))
                    .collect();
                game.act(seat, plays[rng.gen_range(0..plays.len())].clone())
                    .unwrap();
                played += 1;
                let saved = serde_json::to_value(&game).unwrap();
                game = serde_json::from_value(saved.clone()).unwrap();
                assert_eq!(serde_json::to_value(&game).unwrap(), saved);
            }
            let View::TheCrew(view) = game.view(0) else {
                unreachable!()
            };
            assert_eq!(played, if players == 3 { 39 } else { 40 });
            assert_eq!(view.tricks_won.iter().sum::<usize>(), 40 / players);
            assert!(view.trick.is_empty());
            assert!(game.outcome().is_none());
            assert_eq!(
                game.legal_actions(0),
                vec![
                    Action::TheCrew(CrewAction::Finish { won: true }),
                    Action::TheCrew(CrewAction::Finish { won: false })
                ]
            );
            assert!((1..players).all(|p| game.legal_actions(p).is_empty()));
            let won = seed % 2 == 0;
            game.act(0, Action::TheCrew(CrewAction::Finish { won }))
                .unwrap();
            assert_eq!(
                game.outcome().unwrap().winners,
                if won {
                    (0..players).collect::<Vec<_>>()
                } else {
                    vec![]
                }
            );
            let replay = game.replay();
            assert_eq!(replay.version, 1);
            assert_eq!(replay.simulate().unwrap().replay(), replay);
        }
    }
}

#[test]
fn revised_games_reject_old_replays_and_persisted_rules_markers() {
    for kind in [
        GameKind::CrazyEights,
        GameKind::Mancala,
        GameKind::Shithead,
        GameKind::Skyjo,
    ] {
        let game = Match::new(kind, 2, [0; 32]).unwrap();
        let replay = game.replay();
        assert_eq!(replay.version, 4);
        for version in 1..4 {
            let mut old = replay.clone();
            old.version = version;
            if version == 1 {
                old.config = None;
            }
            assert!(old.simulate().is_err());
        }
        for marker in [None, Some(3)] {
            let mut saved = serde_json::to_value(&game).unwrap();
            let session = saved.as_object_mut().unwrap().values_mut().next().unwrap();
            let state = session["state"].as_object_mut().unwrap();
            if let Some(version) = marker {
                state.insert("rules_version".into(), version.into());
            } else {
                state.remove("rules_version");
            }
            assert!(serde_json::from_value::<Match>(saved).is_err());
        }
    }
}

#[test]
fn shithead_bomb_configurations_finish_and_replay_with_every_supported_player_count() {
    use arena_core::{Config, games::shithead::GameConfig};
    for four_card_bombs in [false, true] {
        for players in 2..=5 {
            let config = Config::Shithead(GameConfig { four_card_bombs });
            let mut game =
                Match::with_config(config.clone(), players, [players as u8; 32]).unwrap();
            let mut rng = ChaCha12Rng::from_seed([17; 32]);
            for _ in 0..20000 {
                if game.outcome().is_some() {
                    break;
                }
                let seats = game.turn().players;
                let player = seats[rng.gen_range(0..seats.len())];
                let actions = game.legal_actions(player);
                game.act(player, actions[rng.gen_range(0..actions.len())].clone())
                    .unwrap();
            }
            assert!(
                game.outcome().is_some(),
                "{players} players, bombs {four_card_bombs}"
            );
            assert_eq!(game.config(), config);
            let replay = game.replay();
            assert_eq!(replay.simulate().unwrap().replay(), replay);
        }
    }
}

#[test]
fn crazy_eights_configurations_finish_persist_and_replay() {
    use arena_core::{Config, Replay, games::crazy_eights::GameConfig};
    for bits in 0..16 {
        let config = Config::CrazyEights(GameConfig {
            ace_are_you_strong: bits & 1 != 0,
            jack_skip: bits & 2 != 0,
            ten_reverse: bits & 4 != 0,
            two_draw_two: bits & 8 != 0,
        });
        for players in 2..=6 {
            let mut game = Match::with_config(config.clone(), players, [bits; 32]).unwrap();
            let mut rng = ChaCha12Rng::from_seed([bits + 32; 32]);
            for _ in 0..20000 {
                if game.outcome().is_some() {
                    break;
                }
                let player = game.turn().players[0];
                let actions = game.legal_actions(player);
                game.act(player, actions[rng.gen_range(0..actions.len())].clone())
                    .unwrap();
            }
            assert!(
                game.outcome().is_some(),
                "config {bits}, {players} players stalled"
            );
            let restored: Match =
                serde_json::from_value(serde_json::to_value(&game).unwrap()).unwrap();
            assert_eq!(restored.config(), config);
            let replay: Replay =
                serde_json::from_value(serde_json::to_value(game.replay()).unwrap()).unwrap();
            assert_eq!(replay.version, 4);
            assert_eq!(replay.simulate().unwrap().replay(), replay);
            let mut altered = replay.clone();
            altered.version = 1;
            assert!(altered.simulate().is_err());
            altered.config = Some(GameKind::Mancala.default_config());
            altered.version = 2;
            assert!(altered.simulate().is_err());
        }
    }
}

#[test]
fn legacy_crazy_eights_rules_are_rejected_rather_than_reinterpreted() {
    let game = Match::new(GameKind::CrazyEights, 2, [9; 32]).unwrap();
    let replay = game.replay();
    let json = serde_json::to_value(&replay).unwrap();
    assert!(json.get("config").is_some());
    assert_eq!(replay.version, 4);
    for version in [1, 2, 3] {
        let mut legacy = replay.clone();
        legacy.version = version;
        if version == 1 {
            legacy.config = None;
        }
        assert!(legacy.simulate().is_err());
    }
    let mut legacy = serde_json::to_value(&game).unwrap();
    let session = legacy["CrazyEights"].as_object_mut().unwrap();
    session.remove("config");
    let state = session["state"].as_object_mut().unwrap();
    for key in ["config", "pending_draw", "reversed", "rules_version"] {
        state.remove(key);
    }
    assert!(serde_json::from_value::<Match>(legacy).is_err());
}

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
        GameKind::TempelDesSchreckens,
        GameKind::Mancala,
        GameKind::CrazyEights,
        GameKind::Shithead,
        GameKind::Skyjo,
        GameKind::TicTacToe,
    ] {
        for seed in 0..8 {
            let players = if kind == GameKind::TempelDesSchreckens {
                3
            } else {
                2
            };
            let mut game = Match::new(kind, players, [seed; 32]).unwrap();
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
                game.act(replay.records[0].player, replay.records[0].action.clone())
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
        GameKind::TempelDesSchreckens,
        GameKind::Mancala,
        GameKind::CrazyEights,
        GameKind::Shithead,
        GameKind::Skyjo,
        GameKind::TicTacToe,
    ] {
        for count in [0, 1, kind.max_players() + 1, usize::MAX] {
            assert!(Match::new(kind, count, [0; 32]).is_err());
        }
        let min_players = if kind == GameKind::TempelDesSchreckens {
            3
        } else {
            2
        };
        for count in min_players..=kind.max_players() {
            let mut game = Match::new(kind, count, [count as u8; 32]).unwrap();
            let mut rng = ChaCha12Rng::from_seed([count as u8; 32]);
            for step in 0..20000 {
                if game.outcome().is_some() {
                    break;
                }
                let player = game.turn().players[0];
                let actions = game.legal_actions(player);
                assert!(!actions.is_empty(), "{kind:?} with {count} players stalled");
                if step % 32 == 0 {
                    let saved = serde_json::to_value(&game).unwrap();
                    let restored: Match = serde_json::from_value(saved.clone()).unwrap();
                    assert_eq!(serde_json::to_value(&restored).unwrap(), saved);
                    assert_eq!(restored.legal_actions(player), actions);
                    for viewer in 0..count {
                        assert_eq!(
                            serde_json::to_value(restored.view(viewer)).unwrap(),
                            serde_json::to_value(game.view(viewer)).unwrap()
                        );
                    }
                }
                match game.view(player) {
                    arena_core::View::TempelDesSchreckens(view) => {
                        let hidden = view
                            .cards
                            .iter()
                            .flatten()
                            .filter(|card| card.is_none())
                            .count();
                        assert_eq!(
                            hidden + view.revealed.gold + view.revealed.traps + view.revealed.empty,
                            count * 5
                        );
                    }
                    arena_core::View::Mancala(view) => {
                        assert_eq!(view.board.iter().sum::<u16>(), 48)
                    }
                    arena_core::View::Shithead(view) => {
                        let held: usize = view
                            .players
                            .iter()
                            .map(|cards| cards.count + cards.up.len() + cards.down)
                            .sum();
                        assert_eq!(held + view.stock + view.pile + view.burned, 52);
                    }
                    _ => {}
                }
                game.act(player, actions[rng.gen_range(0..actions.len())].clone())
                    .unwrap();
            }
            assert!(
                game.outcome().is_some(),
                "{kind:?} with {count} players did not finish"
            );
            let replay = game.replay();
            assert_eq!(replay.simulate().unwrap().replay(), replay);
        }
    }
}

#[test]
fn tempel_all_player_counts_keep_hidden_state_private_and_resume_seeded_rounds() {
    use arena_core::games::tempel_des_schreckens::{Action as TempelAction, Phase};
    for players in 3..=10 {
        let mut outcomes = [false; 2];
        for seed in 0..32 {
            let mut game = Match::new(GameKind::TempelDesSchreckens, players, [seed; 32]).unwrap();
            let mut rng = ChaCha12Rng::from_seed([seed + 32; 32]);
            let mut steps = 0;
            while game.outcome().is_none() {
                let seats = game.turn().players;
                let player = seats[rng.gen_range(0..seats.len())];
                let actions = game.legal_actions(player);
                assert!(!actions.is_empty());
                let arena_core::View::TempelDesSchreckens(view) = game.view(player) else {
                    unreachable!()
                };
                let hand_hidden = view.cards[player].iter().filter(|c| c.is_none()).count();
                assert_eq!(
                    hand_hidden,
                    view.hand_counts.gold + view.hand_counts.traps + view.hand_counts.empty
                );
                if view.phase == Phase::Inspect {
                    assert!(view.cards.iter().flatten().all(Option::is_none));
                    assert_eq!(view.cards[0].len(), 6 - view.round);
                }
                let json = serde_json::to_value(&view).unwrap();
                for private in ["hands", "roles", "rng", "seed", "unused_role"] {
                    assert!(json.get(private).is_none());
                }
                let saved = serde_json::to_value(&game).unwrap();
                let mut restored: Match = serde_json::from_value(saved).unwrap();
                let action = actions[rng.gen_range(0..actions.len())].clone();
                game.act(player, action.clone()).unwrap();
                restored.act(player, action).unwrap();
                assert_eq!(
                    serde_json::to_value(&game).unwrap(),
                    serde_json::to_value(restored).unwrap()
                );
                steps += 1;
                assert!(steps <= 1 + players * 8);
            }
            let arena_core::View::TempelDesSchreckens(view) = game.view(0) else {
                unreachable!()
            };
            let won = view.revealed.gold == view.distribution.cards.gold;
            outcomes[usize::from(won)] = true;
            for player in 0..players {
                let arena_core::View::TempelDesSchreckens(personal) = game.view(player) else {
                    unreachable!()
                };
                let winner = if won {
                    arena_core::games::tempel_des_schreckens::Role::Adventurer
                } else {
                    arena_core::games::tempel_des_schreckens::Role::Guardian
                };
                assert_eq!(
                    game.outcome().unwrap().winners.contains(&player),
                    personal.role == winner
                );
                assert!(game.legal_actions(player).is_empty());
            }
            let replay = game.replay();
            assert_eq!(replay.version, 1);
            assert_eq!(replay.simulate().unwrap().replay(), replay);
            let before = serde_json::to_value(&game).unwrap();
            assert!(
                game.act(0, Action::TempelDesSchreckens(TempelAction::Ready))
                    .is_err()
            );
            assert_eq!(serde_json::to_value(&game).unwrap(), before);
        }
        assert_eq!(outcomes, [true, true], "{players} players");
    }
}

#[test]
fn player_views_do_not_contain_private_state() {
    for kind in [GameKind::CrazyEights, GameKind::Shithead, GameKind::Skyjo] {
        let game = Match::new(kind, 2, [1; 32]).unwrap();
        let view = serde_json::to_value(game.view(0)).unwrap();
        assert!(view["state"].get("seed").is_none());
        assert!(view["state"].get("deck").is_none());
        assert_eq!(
            game.legal_actions(1).is_empty(),
            !game.turn().players.contains(&1)
        );
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
