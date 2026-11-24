# Frontend design

## Direction

Inkline: a cream canvas, a green playing table, and ink outlines. The playing
surface is the focus; color carries meaning rather than decoration. Playfulness
comes from tactile controls: only pressable elements cast a hard shadow, pressing
moves them onto it, and chosen options stay pressed in.

- Show only what is needed to join, create, or play. The landing page needs no
  branding, headline, explanatory copy, or recent-room list.
- Place the nickname above separate create/join panels. Use two columns on
  desktop and tablet, and one on narrow screens. Invitation links show joining
  only, with the room code filled in.
- Corner buttons sit in the upper corners on larger screens and the lower corners
  on mobile: settings at the end, room controls (during games) at the start. Their
  popovers are dismissible and never shift the page.
- Meaning colors: the highlight marks selection, the active player, and the most
  recent play; the focus color marks keyboard focus. Illegal cards use an opaque
  muted face, never transparency.
- Use comfortable touch targets (at least 44px), visible keyboard focus, and
  placeholder hints backed by accessible field names. Never require hover.
- Avoid ornamental motion, tilted elements, small decorative pills, gradients,
  and glows.

## Themes

`src/style.ts` imports the layout tokens (`design.css`), the active theme, and
element defaults (`base.css`), then sets `<html data-theme>`. A theme is one CSS
file in `src/themes/` that scopes its tokens to `:root[data-theme='<name>']` and
imports its bundled font. It defines every token documented in
`themes/inkline.css`: type, surfaces and text, meaning colors, seat colors,
lines and radii, depth (raised, hover, and pressed shadows and shifts), and card
faces. Components use only these tokens and the layout tokens, so adding a theme
needs no component changes. Fonts are bundled through Fontsource; fonts and
translations require no external requests.

English, German, and German (cool) have independent catalogs. Use the first
supported device language unless the user has saved a selection; English is the
fallback. German (cool) is an explicit selection, not a device locale.

## Room screens

- Lobby (no game): room code, game choice tiles and rules, seats, bench, and the
  host's start button. Other members see the same setup read-only.
- Game: a status strip with the game name and whose turn it is (highlighted on
  the viewer's turn) or the result, then the board. Room code, members, replays,
  and host commands (abort, finish, play again, return to setup) live in the room
  controls popover. Immersive boards (The Crew) fill the viewport and show their
  own status; the popover then repeats the result.

## Playing cards

All cards share a rounded 5:7 shell. Layer symbol/letter cards onto that shell,
then standard playing cards onto the symbol layer. Face-down cards use the shell
without receiving hidden face data. Standard suits use SVG, keeping their rendering
independent of installed fonts.

Cards can be passive displays or native buttons. Accessible names describe their
visible meaning; a selectable button can expose its selected state. Enabled card
buttons are raised; a selected card lifts. Games own their hand/table layouts,
legal-action behavior, and responsive card sizing.

## The Crew

The Crew fills one viewport with a straight, non-overlapping hand along the
bottom, a central trick, and opponent racks around the outside. Cards have
saturated color faces, contrasting numbers, and outline shapes. Illegal cards
use opaque muted colors only on the viewer's turn; card selection is independent
of permission to submit a move. Communication markers sit inside their cards.
Use one comfortable card size in every context, including trick history; let
rows and the table area scroll rather than reduce legibility. In portrait mobile,
place the viewer's communication and won tricks above the full-width hand.
Use a prominent “Your turn” status for the active viewer, not their own nickname.

Keep the playing space free of room headings and setup lists. Communication,
trick history, and player details open in dismissible popovers without shifting
the hand. Long hands scroll horizontally; the page itself does not scroll during
play.
