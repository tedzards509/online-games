import type { Schemas } from '../../lib/types';

/** Printed Crew colors and redundant shapes; black cards are rocket trumps. */
export const crewColors = {
  blue: '#008bcf',
  green: '#8cc63e',
  pink: '#e77cb1',
  yellow: '#dfae41',
  black: '#242825',
} satisfies Record<Schemas['CrewColor'], string>;

/** Card identity, independent of its changing hand index. */
export function cardKey(card: Schemas['CrewCard']): string {
  return `${card.color}-${card.number}`;
}
