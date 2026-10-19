//! Checks that custom action, event, and observation types work with sessions.

use arena_core::{Error, Game, GameEvent, Outcome, Player, Resolution, Session, Turn, game::Seed};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
enum CustomEvent {
    Changed,
    Ended(Outcome),
}

impl GameEvent for CustomEvent {
    fn finished(outcome: Outcome) -> Self {
        Self::Ended(outcome)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Counter(bool);

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
struct CounterConfig {
    finished: bool,
}

impl Game for Counter {
    type Config = CounterConfig;
    type Action = bool;
    type Event = CustomEvent;
    type View = bool;
    fn with_config(players: usize, _: Seed, config: Self::Config) -> Result<Self, Error> {
        if players != 2 {
            return Err(Error::PlayerCount);
        }
        Ok(Self(config.finished))
    }
    fn turn(&self) -> Turn {
        Turn::first()
    }
    fn outcome(&self) -> Option<Outcome> {
        self.0.then_some(Outcome {
            winners: vec![0],
            scores: None,
        })
    }
    fn legal_actions(&self, player: Player) -> Vec<bool> {
        if self.0 || player != 0 {
            vec![]
        } else {
            vec![true]
        }
    }
    fn advance(&mut self, _: Player, action: &bool) -> Vec<CustomEvent> {
        self.0 = *action;
        vec![CustomEvent::Changed]
    }
    fn view(&self, _: Player) -> bool {
        self.0
    }
    fn resolutions(&self) -> Vec<Resolution> {
        if self.0 {
            vec![]
        } else {
            vec![Resolution::SharedResult { won: true }]
        }
    }
    fn resolve(&mut self, _: Resolution) -> Vec<CustomEvent> {
        self.0 = true;
        vec![CustomEvent::Changed]
    }
}

#[test]
fn external_resolutions_validate_and_record_custom_events_without_an_actor() {
    let mut session = Session::<Counter>::new(2, [0; 32]).unwrap();
    let before = serde_json::to_value(&session).unwrap();
    assert_eq!(
        session
            .resolve(Resolution::SharedResult { won: false })
            .unwrap_err(),
        Error::IllegalMove
    );
    assert_eq!(serde_json::to_value(&session).unwrap(), before);
    let resolution = Resolution::SharedResult { won: true };
    let record = session.resolve(resolution).unwrap();
    assert_eq!(record.resolution, resolution);
    assert_eq!(
        record.events,
        vec![
            CustomEvent::Changed,
            CustomEvent::Ended(Outcome {
                winners: vec![0],
                scores: None
            })
        ]
    );
    assert!(session.records().is_empty());
    let saved = serde_json::to_value(&session).unwrap();
    assert!(saved["resolution"].get("player").is_none());
    assert_eq!(session.resolve(resolution).unwrap_err(), Error::Finished);
    assert_eq!(session.act(0, true).unwrap_err(), Error::Finished);
    assert_eq!(serde_json::to_value(&session).unwrap(), saved);
    let restored: Session<Counter> = serde_json::from_value(saved).unwrap();
    assert_eq!(restored.resolution(), session.resolution());
    let mut automatic = Session::<Reaction>::new(3, [0; 32]).unwrap();
    let before = serde_json::to_value(&automatic).unwrap();
    assert_eq!(
        automatic.resolve(resolution).unwrap_err(),
        Error::IllegalMove
    );
    assert_eq!(serde_json::to_value(&automatic).unwrap(), before);
}

#[test]
fn configuration_controls_setup_and_survives_serialization() {
    let config = CounterConfig { finished: true };
    let session = Session::<Counter>::with_config(2, [0; 32], config.clone()).unwrap();
    assert!(session.state().view(0));
    assert_eq!(session.config(), &config);
    let restored: Session<Counter> =
        serde_json::from_value(serde_json::to_value(&session).unwrap()).unwrap();
    assert_eq!(restored.config(), &config);
    assert!(restored.state().view(0));
    let mut legacy = serde_json::to_value(Session::<Counter>::new(2, [0; 32]).unwrap()).unwrap();
    legacy.as_object_mut().unwrap().remove("config");
    let restored: Session<Counter> = serde_json::from_value(legacy).unwrap();
    assert_eq!(restored.config(), &CounterConfig::default());
}

#[test]
fn sessions_accept_independent_action_event_and_view_types() {
    let mut session = Session::<Counter>::new(2, [0; 32]).unwrap();
    assert_eq!(session.act(0, false).unwrap_err(), Error::IllegalMove);
    let record = session.act(0, true).unwrap();
    assert_eq!(
        record.events,
        vec![
            CustomEvent::Changed,
            CustomEvent::Ended(Outcome {
                winners: vec![0],
                scores: None
            })
        ]
    );
    let restored: Session<Counter> =
        serde_json::from_value(serde_json::to_value(session).unwrap()).unwrap();
    assert!(restored.state().view(0));
    assert_eq!(restored.records().len(), 1);
}

#[derive(Clone, Serialize, Deserialize)]
struct Reaction {
    turn: Turn,
    winner: Option<Player>,
}

impl Game for Reaction {
    type Config = arena_core::game::GameConfig;
    type Action = usize;
    type Event = CustomEvent;
    type View = Option<Player>;
    fn with_config(players: usize, _: Seed, _: Self::Config) -> Result<Self, Error> {
        if players != 3 {
            return Err(Error::PlayerCount);
        }
        Ok(Self {
            turn: Turn {
                players: vec![0, 1, 2],
                number: 0,
            },
            winner: None,
        })
    }
    fn turn(&self) -> Turn {
        self.turn.clone()
    }
    fn outcome(&self) -> Option<Outcome> {
        self.winner.map(|player| Outcome {
            winners: vec![player],
            scores: None,
        })
    }
    fn legal_actions(&self, player: Player) -> Vec<usize> {
        if self.winner.is_none() && self.turn.players.contains(&player) {
            vec![player + 10]
        } else {
            vec![]
        }
    }
    fn advance(&mut self, player: Player, _: &usize) -> Vec<CustomEvent> {
        if self.turn.number == 0 {
            self.turn.players = vec![(player + 1) % 3];
        } else {
            self.winner = Some(player);
        }
        self.turn.number += 1;
        vec![CustomEvent::Changed]
    }
    fn view(&self, _: Player) -> Self::View {
        self.winner
    }
}

#[test]
fn any_eligible_seat_can_act_then_only_the_next_seat_can_respond() {
    for player in 0..3 {
        let mut game = Session::<Reaction>::new(3, [0; 32]).unwrap();
        let before = serde_json::to_value(&game).unwrap();
        assert_eq!(
            game.act(player, (player + 1) % 3 + 10).unwrap_err(),
            Error::IllegalMove
        );
        assert_eq!(game.act(3, 13).unwrap_err(), Error::NotYourTurn);
        assert_eq!(serde_json::to_value(&game).unwrap(), before);
        game.act(player, player + 10).unwrap();
        let next = (player + 1) % 3;
        assert_eq!(game.state().turn().players, vec![next]);
        let before = serde_json::to_value(&game).unwrap();
        assert!(game.state().legal_actions(player).is_empty());
        assert_eq!(
            game.act(player, player + 10).unwrap_err(),
            Error::NotYourTurn
        );
        assert_eq!(serde_json::to_value(&game).unwrap(), before);
        game.act(next, next + 10).unwrap();
        let mut replayed = Session::<Reaction>::new(3, [0; 32]).unwrap();
        for record in game.records() {
            replayed.act(record.player, record.action).unwrap();
        }
        assert_eq!(replayed.records(), game.records());
        assert_eq!(replayed.state().view(0), Some(next));
    }
}

#[test]
fn turn_advance_requires_exactly_one_player_and_preserves_state_on_panic() {
    let mut turn = Turn {
        players: vec![2],
        number: 7,
    };
    turn.advance(3);
    assert_eq!(
        turn,
        Turn {
            players: vec![0],
            number: 8
        }
    );
    for players in [vec![], vec![0, 1]] {
        let mut turn = Turn { players, number: 7 };
        let before = turn.clone();
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| turn.advance(3))).is_err()
        );
        assert_eq!(turn, before);
    }
}

#[test]
fn legacy_persisted_turns_load_as_singleton_lists() {
    let turn: Turn = serde_json::from_str(r#"{"player":1,"number":7}"#).unwrap();
    assert_eq!(
        turn,
        Turn {
            players: vec![1],
            number: 7
        }
    );
    assert_eq!(
        serde_json::to_value(turn).unwrap(),
        serde_json::json!({"players":[1],"number":7})
    );
}
