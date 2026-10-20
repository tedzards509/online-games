import type { Schemas } from '../../lib/types';

/** Printed Crew colors and redundant shapes; black trumps use a jellyfish. */
export const crewColors = {
  blue: '#008bcf',
  green: '#8cc63e',
  pink: '#e77cb1',
  yellow: '#dfae41',
  black: '#242825',
} satisfies Record<Schemas['CrewColor'], string>;

/** At least 4.5:1 text contrast on all four printed color faces. */
export const crewInk = '#0e1e16';

/** Card identity, independent of its changing hand index. */
export function cardKey(card: Schemas['CrewCard']): string {
  return `${card.color}-${card.number}`;
}

/** Winning seat in a public, possibly incomplete trick. */
export function trickLeader(cards: Schemas['CrewPlay'][]): number | null {
  let best = cards[0];
  if (!best) return null;
  for (const played of cards.slice(1)) {
    if (
      (played.card.color === best.card.color &&
        played.card.number > best.card.number) ||
      (played.card.color === 'black' && best.card.color !== 'black')
    ) {
      best = played;
    }
  }
  return best.player;
}
