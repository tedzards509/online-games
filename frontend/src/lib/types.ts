import type { components } from './api.generated';

export type Schemas = components['schemas'];
export type Snapshot = Schemas['Snapshot'];
export type GameSnapshot = Schemas['GameSnapshot'];
export type Membership = Schemas['Membership'];
export type Action = Schemas['Action'];
export type GameKind = Schemas['GameKind'];
export type Card = Schemas['Card'];
export type Suit = Schemas['Suit'];
export type ClientMessage = Schemas['ClientMessage'];
export type ServerMessage = Schemas['ServerMessage'];

/** Selectable games in display order; names are `gameNames.<kind>`. */
export const games: GameKind[] = [
  'tempel_des_schreckens',
  'the_crew',
  'mancala',
  'crazy_eights',
  'shithead',
  'skyjo',
  'tic_tac_toe',
];

export const suits: Record<Suit, string> = {
  clubs: '♣',
  diamonds: '♦',
  hearts: '♥',
  spades: '♠',
};

/** Orders standard cards by rank (2 → ace), then suit, for sorted hands. */
export function byRank(a: Card, b: Card): number {
  return a.rank - b.rank || a.suit.localeCompare(b.suit);
}

/** Props every game board receives from GameBoard. Boards may require more. */
export type BoardProps<View> = {
  view: View;
  /** The viewer's legal actions; empty while a command is pending or offline. */
  actions: Action[];
  /** Nicknames by player index. */
  names: string[];
  /** The viewer's player index. */
  you: number;
  /** Player indices that may act now; empty after the game ends. */
  active: number[];
  finished: boolean;
  move: (action: Action) => void;
};
