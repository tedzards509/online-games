# Frontend

Svelte 5 SPA. Run and verification commands are in the [root README](../README.md).
[Design direction](../docs/frontend-design.md) describes the visual intent;
[`src/design.css`](src/design.css) defines the shared values with usage comments.

## Shared UI

- `components/ui/Button.svelte`: native button attributes and a children snippet;
  `variant="primary"` (default) or `"secondary"`. Defaults to `type="button"`;
  explicitly use `type="submit"` in forms.
- `components/ui/TextField.svelte`: native input attributes, bindable `value`, and
  required `label`, used as both placeholder and accessible name.
- `components/ui/Panel.svelte`: cream container; its parent owns layout.
- `components/LanguageSettings.svelte`: native popover with Escape/outside-click
  dismissal and a native language select. Uses a 40rem mobile breakpoint.

Shared controls use touch-sized heights and visible focus outlines. The landing
layout changes at 40rem; tablet and desktop widths use two equal panels. Tokens
are globally available, but shared component styles are scoped. Existing game
boards do not use these components yet.

## Language

`lib/i18n.ts` provides the reactive `t` and `locale` stores and `setLanguage`.
i18next resolves messages and interpolation; Svelte escapes rendered text.

```svelte
<script lang="ts">
  import { t } from './lib/i18n';
</script>
<span>{$t('joinRoom')}</span>
```

Edit `lib/locales/en.json`, `de.json`, and `de-cool.json` independently, keeping
their key structures aligned. Do not alias German (cool) to German. Device
preferences are matched in order by base language (`de-AT` → `de`); otherwise
English is used. An explicit choice is saved under `arena-language` in browser
storage. German (cool) sets the document's language to `de` for assistive tools.
Landing errors use translated copy; server diagnostics are not translation keys.
The Crew board uses the `crew` catalog namespace. Other room/game copy remains
English.

## Fonts

`main.ts` imports Nunito Sans through `@fontsource-variable/nunito-sans` (OFL-1.1).
Vite emits local WOFF2 assets; no Google Fonts stylesheet or font request is used.
`public/fonts/nunito-sans-OFL.txt` distributes its license. Standard suit drawings and the settings icon
are inline SVG rather than icon fonts.

## Playing cards

`components/cards/` contains composable layers:

| Component | Content and props |
| --- | --- |
| `PlayingCard` | Shape only; required `label` and children snippet. Arbitrary SVG/image/markup content. |
| `SymbolCard` | Uses `PlayingCard`; required `rank` and `label`, optional text `symbol` or `symbolContent` snippet. Mirrored corner indices and a central symbol. |
| `StandardPlayingCard` | Uses `SymbolCard`; required typed `card: { rank, suit }`. Derives face letters, suit SVG/color, and translated accessible name. Optional `label` override. |
| `CardBack` | Uses `PlayingCard`; face-down pattern and translated name. Accepts no face data. |

All layers accept `width` (CSS length), `background`, `onclick`, `disabled`, and
`selected`; the shell and symbol layer also accept `color`. Standard cards derive
their own suit color. `width` defaults to `--arena-card-width`; corners and symbols
scale with the card using container units. Games choose card width and hand layout.
Image snippets should constrain their own content to fit the shell.

Without `onclick`, a card is a non-focusable image with an accessible name.
With `onclick`, it becomes a native button; `selected` exposes `aria-pressed` when
provided. Its internal graphics are hidden from assistive tools to avoid duplicate
announcements. Never place interactive controls inside card content.

```svelte
<StandardPlayingCard card={{ rank: 14, suit: 'hearts' }} />
<SymbolCard rank="7" symbol="★" label="Yellow seven" color="#765900" />
<CardBack width="3.5rem" />
<PlayingCard label="Moon">
  <svg viewBox="0 0 32 32" width="80%" aria-hidden="true">
    <path d="M24 25A13 13 0 0 1 10 3a12 12 0 0 0 14 22" fill="currentColor" />
  </svg>
</PlayingCard>
```

## The Crew table

`components/TheCrew.svelte` owns the viewport layout, card selection, legal command
lookup, and stable visual trick offsets. `components/crew/` supplies its card face,
communication slot, trick pile, and opponent/info panel. Room metadata uses
member IDs; game actions and marker arrays use frozen player indices.

- `crew/cards.ts` documents the exact printed colors. SVG circle, triangle,
  square, diagonal cross, and rocket outlines distinguish suits without relying
  on color. Faces use the printed color; numbers and outlines use contrasting ink.
- Own cards form a non-overlapping row. Long hands scroll horizontally instead
  of shrinking touch targets. Cards stay selectable outside the viewer's turn
  and while a command is pending; only command buttons are gated. Illegal plays
  on the viewer's turn use an opaque muted face, never transparency. Selection
  survives rejected commands and follows card identity, not its changing index.
- Opponent back rows render only public hand counts with shared card backs. Player
  info contains public room identity, seat, hand/trick counts, communication, and
  commander/next-player status. It never receives another player's hand.
- Communication markers show unused/active in green and played in red. Active
  markers sit on the card's horizontal centre at top, middle, or bottom for
  highest, only, or lowest.
  `communication_played` and `commander` come from every server projection;
  neither marker relies on a browser remembering earlier tricks.
- Table-specific dimensions live in the board's scoped CSS. `--arena-card-width`
  is `4.5rem` throughout The Crew (72 × 101px at default text size), including
  hands, backs, communications, piles, and both trick views. Card rows and the
  table area scroll when needed; card faces do not shrink to fit.
  In portrait mobile, the viewer's communication and pile sit above a full-width
  hand. Desktop and landscape keep them beside the hand. The main playing
  space is exactly one viewport (`100vh`, using `100dvh` on supported browsers).
  Communication and last-trick popovers do
  not resize the table. The host's room-controls button mirrors the settings
  button; setup, finish, abort/rematch, room code, and replay controls live there.
  Disconnections remain visible as a compact status notice.
- The previous-trick popover displays card faces with player captions and the
  winner. A high-contrast localized turn pill identifies the viewer's actual
  card-playing turn (`next_player`), not eligibility to communicate.
