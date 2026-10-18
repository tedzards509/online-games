//! Checks that custom action, event, and observation types work with sessions.

use arena_core::{Error, Game, GameEvent, Outcome, Player, Session, Turn, game::Seed};
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

impl Game for Counter {
    type Action = bool;
    type Event = CustomEvent;
    type View = bool;
    fn new(players: usize, _: Seed) -> Result<Self, Error> {
        if players != 2 {
            return Err(Error::PlayerCount);
        }
        Ok(Self(false))
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
    fn legal_actions(&self) -> Vec<bool> {
        if self.0 { vec![] } else { vec![true] }
    }
    fn advance(&mut self, action: &bool) -> Vec<CustomEvent> {
        self.0 = *action;
        vec![CustomEvent::Changed]
    }
    fn view(&self, _: Player) -> bool {
        self.0
    }
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
