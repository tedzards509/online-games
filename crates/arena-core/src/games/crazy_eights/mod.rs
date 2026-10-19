use crate::{
    cards::{Card, Deck, Suit, standard_deck},
    game::{Error, Event, Game, Outcome, Player, Seed, Turn},
};
use rand::SeedableRng;
use rand_chacha::ChaCha12Rng;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

const WILD_RANK: u8 = 8;

/// Optional rank effects; eights are always wild. All options default to false.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
#[schema(as = CrazyEightsGameConfig)]
pub struct GameConfig {
    pub ace_are_you_strong: bool,
    pub jack_skip: bool,
    pub ten_reverse: bool,
    pub two_draw_two: bool,
}

/// A draw-two obligation: aces permit an ace response; twos do not stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
#[schema(as = CrazyEightsPendingDraw)]
pub enum PendingDraw {
    Ace,
    Two,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
#[schema(as = CrazyEightsAction)]
pub enum Action {
    Play { index: usize, suit: Option<Suit> },
    Draw,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn card(rank: u8, suit: Suit) -> Card {
        Card { rank, suit }
    }

    #[test]
    fn initial_state_deals_seven_or_five_and_opening_card_has_no_effect() {
        for players in 2..=6 {
            let game =
                CrazyEights::with_config(players, [0; 32], configured(players).config).unwrap();
            assert!(
                game.hands
                    .iter()
                    .all(|hand| hand.len() == if players == 2 { 7 } else { 5 })
            );
            assert_eq!(game.turn, Turn::first());
            assert_eq!(game.active_suit, game.top_card().suit);
            assert_eq!(
                game.deck.len()
                    + game.hands.iter().map(Vec::len).sum::<usize>()
                    + game.discard.len(),
                52
            );
            assert_eq!(game.pending_draw, None);
            assert!(!game.reversed);
            assert!(game.outcome.is_none());
        }
    }

    fn configured(players: usize) -> CrazyEights {
        let mut game = CrazyEights::with_config(
            players,
            [0; 32],
            GameConfig {
                ace_are_you_strong: true,
                jack_skip: true,
                ten_reverse: true,
                two_draw_two: true,
            },
        )
        .unwrap();
        game.discard = vec![card(3, Suit::Hearts)];
        game.active_suit = Suit::Hearts;
        game
    }

    fn play(game: &mut CrazyEights, rank: u8) {
        let player = game.turn.players[0];
        game.hands[player] = vec![card(rank, Suit::Hearts), card(4, Suit::Clubs)];
        game.advance(
            player,
            &Action::Play {
                index: 0,
                suit: None,
            },
        );
    }

    #[test]
    fn responding_ace_clears_the_obligation_and_next_ace_asks_again() {
        let mut game = configured(3);
        play(&mut game, 14);
        game.hands[1] = vec![
            card(14, Suit::Spades),
            card(8, Suit::Hearts),
            card(3, Suit::Hearts),
        ];
        assert_eq!(
            game.legal_actions(1),
            vec![
                Action::Play {
                    index: 0,
                    suit: None
                },
                Action::Draw
            ]
        );
        game.advance(
            1,
            &Action::Play {
                index: 0,
                suit: None,
            },
        );
        assert_eq!(game.pending_draw, None);
        assert!(game.legal_actions(2).contains(&Action::Draw));
        play(&mut game, 14);
        assert_eq!(game.pending_draw, Some(PendingDraw::Ace));
        let before = game.hands[0].len();
        game.advance(0, &Action::Draw);
        assert_eq!(game.hands[0].len(), before + 2);
        assert_eq!(game.pending_draw, None);
        assert_eq!(game.turn.players, vec![1]);
    }

    #[test]
    fn ace_can_be_answered_by_drawing_even_with_an_ace_in_hand() {
        let mut game = configured(3);
        play(&mut game, 14);
        game.hands[1] = vec![card(14, Suit::Spades)];
        let before = game.hands[1].len();
        game.advance(1, &Action::Draw);
        assert_eq!(game.hands[1].len(), before + 2);
        assert_eq!(game.pending_draw, None);
        assert_eq!(game.turn.players, vec![2]);
    }

    #[test]
    fn voluntary_draw_is_legal_with_playable_cards_and_ends_turn() {
        use crate::game::Session;
        let mut session = Session::<CrazyEights>::new(2, [0; 32]).unwrap();
        session.state.discard = vec![card(3, Suit::Hearts)];
        session.state.active_suit = Suit::Hearts;
        session.state.hands[0] = vec![card(5, Suit::Hearts), card(8, Suit::Clubs)];
        let actions = session.state.legal_actions(0);
        assert!(actions.contains(&Action::Play {
            index: 0,
            suit: None
        }));
        assert!(actions.contains(&Action::Draw));
        let before = session.state.hands[0].len();
        session.act(0, Action::Draw).unwrap();
        assert_eq!(session.state.hands[0].len(), before + 1);
        assert_eq!(session.state.turn.players, vec![1]);
        assert!(session.state.legal_actions(0).is_empty());
    }

    #[test]
    fn twos_force_drawing_and_recycle_even_between_penalty_cards() {
        let mut game = configured(3);
        play(&mut game, 2);
        game.hands[1] = vec![
            card(2, Suit::Hearts),
            card(14, Suit::Hearts),
            card(8, Suit::Hearts),
        ];
        assert_eq!(game.legal_actions(1), vec![Action::Draw]);
        game.deck = Deck::new(vec![card(5, Suit::Clubs)]);
        game.advance(1, &Action::Draw);
        assert_eq!(game.hands[1].len(), 5);
        assert_eq!(game.discard, vec![card(2, Suit::Hearts)]);
        assert_eq!(game.turn.players, vec![2]);
        assert_eq!(game.pending_draw, None);
    }

    #[test]
    fn exhausted_penalties_count_as_one_blocked_turn() {
        let mut game = configured(2);
        game.pending_draw = Some(PendingDraw::Two);
        game.deck = Deck::new(vec![]);
        game.hands = vec![vec![card(5, Suit::Clubs)], vec![card(6, Suit::Clubs)]];
        game.advance(0, &Action::Draw);
        assert_eq!(game.consecutive_passes, 1);
        assert!(game.outcome.is_none());
        game.advance(1, &Action::Draw);
        assert!(game.outcome.unwrap().winners.is_empty());
    }

    #[test]
    fn skips_and_reverses_follow_direction_and_increment_one_turn() {
        let mut game = configured(4);
        play(&mut game, 11);
        assert_eq!(game.turn.players, vec![2]);
        play(&mut game, 10);
        assert!(game.reversed);
        assert_eq!(game.turn.players, vec![1]);
        play(&mut game, 11);
        assert_eq!(game.turn.players, vec![3]);
        play(&mut game, 10);
        assert!(!game.reversed);
        assert_eq!(game.turn.players, vec![0]);
        assert_eq!(game.turn.number, 4);
        let mut game = configured(2);
        play(&mut game, 10);
        assert_eq!(game.turn.players, vec![0]);
        play(&mut game, 11);
        assert_eq!(game.turn.players, vec![0]);
    }

    #[test]
    fn disabled_special_ranks_are_ordinary_but_eights_remain_wild() {
        let mut game = configured(3);
        game.config = GameConfig::default();
        game.hands[0] = vec![card(8, Suit::Clubs), card(8, Suit::Hearts)];
        assert_eq!(game.legal_actions(0).len(), 9);
        for index in 0..2 {
            for suit in Suit::ALL {
                assert!(game.legal_actions(0).contains(&Action::Play {
                    index,
                    suit: Some(suit)
                }));
            }
            assert!(
                !game
                    .legal_actions(0)
                    .contains(&Action::Play { index, suit: None })
            );
        }
        for rank in [14, 11, 10, 2] {
            game.turn = Turn::first();
            play(&mut game, rank);
            assert_eq!(game.turn.players, vec![1]);
            assert_eq!(game.pending_draw, None);
            assert!(!game.reversed);
        }
    }

    #[test]
    fn configured_rejections_are_immutable_and_last_special_card_wins() {
        use crate::game::Session;
        let mut session =
            Session::<CrazyEights>::with_config(2, [0; 32], configured(2).config).unwrap();
        session.state.pending_draw = Some(PendingDraw::Two);
        let before = serde_json::to_value(&session).unwrap();
        for action in [
            Action::Play {
                index: usize::MAX,
                suit: None,
            },
            Action::Play {
                index: 0,
                suit: Some(Suit::Clubs),
            },
        ] {
            assert_eq!(session.act(0, action).unwrap_err(), Error::IllegalMove);
            assert_eq!(serde_json::to_value(&session).unwrap(), before);
        }
        for rank in [14, 11, 10, 2, 8] {
            let mut game = configured(2);
            game.hands[0] = vec![card(rank, Suit::Hearts)];
            game.advance(
                0,
                &Action::Play {
                    index: 0,
                    suit: (rank == 8).then_some(Suit::Clubs),
                },
            );
            assert_eq!(game.outcome.as_ref().unwrap().winners, vec![0]);
            assert!(game.legal_actions(1).is_empty());
        }
    }

    #[test]
    fn eight_changes_suit_and_other_special_ranks_do_not() {
        let mut game = CrazyEights::new(2, [0; 32]).unwrap();
        game.hands[0] = vec![card(8, Suit::Clubs), card(3, Suit::Clubs)];
        game.discard = vec![card(14, Suit::Hearts)];
        game.active_suit = Suit::Hearts;
        assert_eq!(game.legal_actions(game.turn.players[0]).len(), 5);
        game.advance(
            game.turn.players[0],
            &Action::Play {
                index: 0,
                suit: Some(Suit::Diamonds),
            },
        );
        assert_eq!(game.active_suit, Suit::Diamonds);
        game.hands[1] = vec![card(2, Suit::Diamonds), card(11, Suit::Spades)];
        assert_eq!(
            game.legal_actions(game.turn.players[0]),
            vec![
                Action::Play {
                    index: 0,
                    suit: None
                },
                Action::Draw
            ]
        );
        game.advance(
            game.turn.players[0],
            &Action::Play {
                index: 0,
                suit: None,
            },
        );
        assert_eq!(game.turn.players, vec![0]);
    }

    #[test]
    fn exhausted_stock_passes_until_everyone_is_blocked() {
        let mut game = CrazyEights::new(2, [0; 32]).unwrap();
        game.deck = Deck::new(vec![]);
        game.discard = vec![card(3, Suit::Hearts)];
        game.active_suit = Suit::Hearts;
        game.hands = vec![vec![card(4, Suit::Clubs)], vec![card(5, Suit::Spades)]];
        assert_eq!(game.legal_actions(game.turn.players[0]), vec![Action::Draw]);
        game.advance(game.turn.players[0], &Action::Draw);
        assert!(game.outcome.is_none());
        game.advance(game.turn.players[0], &Action::Draw);
        assert!(game.outcome.unwrap().winners.is_empty());
    }

    #[test]
    fn drawing_recycles_below_the_top_and_ends_the_turn() {
        let mut game = CrazyEights::new(2, [0; 32]).unwrap();
        let recycled = card(4, Suit::Hearts);
        let top = card(3, Suit::Hearts);
        game.deck = Deck::new(vec![]);
        game.discard = vec![recycled, top];
        game.active_suit = Suit::Hearts;
        game.hands[0] = vec![card(5, Suit::Clubs)];
        game.consecutive_passes = 1;

        assert_eq!(game.legal_actions(0), vec![Action::Draw]);
        game.advance(0, &Action::Draw);

        assert_eq!(game.discard, vec![top]);
        assert_eq!(game.hands[0], vec![card(5, Suit::Clubs), recycled]);
        assert_eq!(game.consecutive_passes, 0);
        assert_eq!(game.turn.players, vec![1]);
        assert!(game.outcome.is_none());
    }

    #[test]
    fn playing_the_last_card_wins_and_resets_passes() {
        let mut game = CrazyEights::new(2, [0; 32]).unwrap();
        game.hands[0] = vec![card(WILD_RANK, Suit::Clubs)];
        game.consecutive_passes = 1;
        game.advance(
            0,
            &Action::Play {
                index: 0,
                suit: Some(Suit::Spades),
            },
        );
        assert_eq!(game.active_suit, Suit::Spades);
        assert_eq!(game.consecutive_passes, 0);
        assert_eq!(game.outcome.as_ref().unwrap().winners, vec![0]);
        assert!(game.legal_actions(1).is_empty());
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrazyEights {
    // Required on deserialization: pre-v3 sessions must not adopt new rules silently.
    rules_version: u32,
    #[serde(default)]
    config: GameConfig,
    #[serde(default)]
    reversed: bool,
    #[serde(default)]
    pending_draw: Option<PendingDraw>,
    hands: Vec<Vec<Card>>,
    deck: Deck<Card>,
    discard: Vec<Card>,
    #[serde(rename = "suit")]
    active_suit: Suit,
    rng: ChaCha12Rng,
    turn: Turn,
    #[serde(rename = "blocked")]
    consecutive_passes: usize,
    outcome: Option<Outcome>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[schema(as = CrazyEightsView)]
pub struct View {
    pub hand: Vec<Card>,
    pub counts: Vec<usize>,
    pub top: Card,
    pub suit: Suit,
    pub stock: usize,
    pub reversed: bool,
    pub pending_draw: Option<PendingDraw>,
}

impl CrazyEights {
    fn top_card(&self) -> Card {
        *self
            .discard
            .last()
            .expect("the discard always has a top card")
    }

    fn play_card(&mut self, player: Player, index: usize, chosen_suit: Option<Suit>) {
        let card = self.hands[player].remove(index);
        self.active_suit = chosen_suit.unwrap_or(card.suit);
        self.discard.push(card);
        self.consecutive_passes = 0;
        self.pending_draw = match card.rank {
            14 if self.config.ace_are_you_strong && self.pending_draw != Some(PendingDraw::Ace) => {
                Some(PendingDraw::Ace)
            }
            2 if self.config.two_draw_two => Some(PendingDraw::Two),
            _ => None,
        };
        if self.hands[player].is_empty() {
            self.outcome = Some(Outcome {
                winners: vec![player],
                scores: None,
            });
        }
    }

    fn draw_or_pass(&mut self, player: Player) {
        let count = if self.pending_draw.take().is_some() {
            2
        } else {
            1
        };
        let mut drawn = 0;
        for _ in 0..count {
            self.deck.recycle(&mut self.discard, &mut self.rng);
            if let Some(card) = self.deck.draw() {
                self.hands[player].push(card);
                drawn += 1;
            }
        }
        if drawn > 0 {
            self.consecutive_passes = 0;
        } else {
            self.consecutive_passes += 1;
        }
        if self.consecutive_passes == self.hands.len() {
            self.outcome = Some(Outcome {
                winners: vec![],
                scores: None,
            });
        }
    }
}

impl Game for CrazyEights {
    type Config = GameConfig;
    type Action = Action;
    type Event = Event;
    type View = View;
    fn with_config(players: usize, seed: Seed, config: Self::Config) -> Result<Self, Error> {
        if !(2..=6).contains(&players) {
            return Err(Error::PlayerCount);
        }
        let mut rng = ChaCha12Rng::from_seed(seed);
        let mut deck = standard_deck();
        deck.shuffle(&mut rng);
        let cards_per_player = if players == 2 { 7 } else { 5 };
        let hands = (0..players)
            .map(|_| {
                (0..cards_per_player)
                    .map(|_| deck.draw().expect("the deck can deal every hand"))
                    .collect()
            })
            .collect();
        let top = deck.draw().expect("the deal leaves an opening discard");
        Ok(Self {
            rules_version: 3,
            config,
            reversed: false,
            pending_draw: None,
            hands,
            deck,
            discard: vec![top],
            active_suit: top.suit,
            rng,
            turn: Turn::first(),
            consecutive_passes: 0,
            outcome: None,
        })
    }
    fn turn(&self) -> Turn {
        self.turn.clone()
    }
    fn outcome(&self) -> Option<Outcome> {
        self.outcome.clone()
    }
    fn legal_actions(&self, player: Player) -> Vec<Action> {
        if self.outcome.is_some() || !self.turn.players.contains(&player) {
            return vec![];
        }
        let top = self.top_card();
        let mut actions = vec![];
        for (index, card) in self.hands[player].iter().enumerate() {
            if let Some(pending) = self.pending_draw {
                if pending == PendingDraw::Ace && card.rank == 14 {
                    actions.push(Action::Play { index, suit: None });
                }
            } else if card.rank == WILD_RANK {
                actions.extend(Suit::ALL.map(|suit| Action::Play {
                    index,
                    suit: Some(suit),
                }));
            } else if card.suit == self.active_suit || card.rank == top.rank {
                actions.push(Action::Play { index, suit: None });
            }
        }
        actions.push(Action::Draw);
        actions
    }
    fn advance(&mut self, player: Player, action: &Action) -> Vec<Event> {
        let mut steps = 1;
        match *action {
            Action::Play { index, suit } => {
                let rank = self.hands[player][index].rank;
                self.play_card(player, index, suit);
                if rank == 10 && self.config.ten_reverse {
                    self.reversed = !self.reversed;
                    if self.hands.len() == 2 {
                        steps = 2;
                    }
                }
                if rank == 11 && self.config.jack_skip {
                    steps = 2;
                }
            }
            Action::Draw => self.draw_or_pass(player),
        }
        let count = self.hands.len();
        let offset = if self.reversed { count - steps } else { steps };
        self.turn.players = vec![(player + offset) % count];
        self.turn.number += 1;
        vec![Event::Played { player }]
    }
    fn view(&self, viewer: Player) -> View {
        View {
            hand: self.hands.get(viewer).cloned().unwrap_or_default(),
            counts: self.hands.iter().map(Vec::len).collect(),
            top: self.top_card(),
            suit: self.active_suit,
            stock: self.deck.len(),
            reversed: self.reversed,
            pending_draw: self.pending_draw,
        }
    }
}
