# Frontend

Svelte 5 SPA. Run and verification commands are in the [root README](../README.md).
[Design direction](../docs/frontend-design.md) describes the visual intent;
[`src/design.css`](src/design.css) defines the shared values with usage comments.

## Shared UI

`components/ui/` holds themed controls; each scopes its styles and uses only
design tokens:

| Component | Use |
| --- | --- |
| `Button` | Native button attributes and children. `variant` `primary` (default), `secondary`, or `quiet`; `size` `small` (44px), `regular`, or `large`; `square` for icon-only buttons with an `aria-label`. Defaults to `type="button"`. |
| `TextField` | Native input attributes, bindable `value`, and `label` used as placeholder and accessible name. |
| `ChoiceGroup` | Radio tiles with a visible legend; the chosen tile stays pressed in. |
| `Checkbox`, `Select` | Labelled native controls with `onchange(value)`. |
| `Panel` | Outlined surface that stacks its children; accepts section attributes. |
| `Corner`, `Popover` | Corner-fixed trigger placement and the matching dismissible native popover (`start` or `end`). |
| `Notice` | Inline status, or an alert with `tone="error"`. |

`components/table/` holds shared game widgets: `Board` (every board's root),
`PlayerBar` (avatar, name, public details, `active` highlight, `data-active`),
`PlayerList` (labelled list of player bars with a details snippet), `Avatar`,
`TableSurface` (the green playing area), `CardRow` (labelled, horizontally
scrolling hand), `Pile` (card with caption), `BoardSpace` (square cell or round
pit, raised while it accepts a move), and `Actions` (command button row). Boards receive `BoardProps` (`lib/types.ts`):
view, legal actions, names, viewer index, active players, finished flag, and
`move`.

`components/room/` holds the lobby, room controls popover, game status strip,
and their parts. Breakpoints use 40rem.

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
The Crew board uses the `crew` catalog namespace. Room and other game copy is
English.

## Fonts

Each theme imports its font through Fontsource; Inkline uses Bricolage Grotesque
(`@fontsource-variable/bricolage-grotesque`, OFL-1.1). Vite emits local WOFF2
assets; no external font request is used. `public/fonts/` distributes the
license. Standard suit drawings and icons are inline SVG rather than icon fonts.

## Playing cards

`components/cards/` contains composable layers:

| Component | Content and props |
| --- | --- |
| `PlayingCard` | Shape only; required `label` and children snippet. Arbitrary SVG/image/markup content. |
| `SymbolCard` | Uses `PlayingCard`; required `rank` and `label`, optional text `symbol` or `symbolContent` snippet. Mirrored corner indices and a central symbol. |
| `StandardPlayingCard` | Uses `SymbolCard`; required typed `card: { rank, suit }`. Derives face letters, suit SVG/color, and translated accessible name. Optional `label` override. |
| `CardBack` | Uses `PlayingCard`; face-down pattern and translated name. Accepts no face data. |
| `CardSlot` | Card-sized placeholder: dashed outline (empty pile) or an invisible gap (`outlined={false}`). |

All layers accept `width` (CSS length), `background`, `onclick`, `disabled`,
`selected`, `muted` (opaque illegal-play face), and `highlighted` (e.g. the most
recent play); the shell and symbol layer also accept `color`. Standard cards derive
their own suit color. `width` defaults to `--arena-card-width`
(`--arena-card-width-small` for secondary cards); corners and symbols
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
  square, diagonal cross, and jellyfish outlines distinguish suits without relying
  on color. Faces use the printed color; numbers and outlines use contrasting ink.
- Own cards form a non-overlapping row. Long hands scroll horizontally instead
  of shrinking touch targets. Cards stay selectable outside the viewer's turn
  and while a command is pending; only command buttons are gated. Illegal plays
  on the viewer's turn use an opaque muted face, never transparency. Selection
  survives rejected commands and follows card identity, not its changing index.
- Opponent back rows render only public hand counts with shared card backs. Player
  info contains public room identity, seat, hand/trick counts, communication, and
  commander/next-player status. It never receives another player's hand.
  Info buttons fit their name and icon; opponent commander markers occupy a
  separate row below the communication and won-trick rack.
- Communication markers show unused/active in green and played in red. Active
  markers have half a card's width as their diameter; commander markers have
  three quarters of a card's width as their diameter.
  Communication markers sit on the card's horizontal centre at top, middle, or bottom for
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
  not resize the table. The room-controls button mirrors the settings button for
  every member; finish, abort/rematch, and other host commands live there.
  Disconnections remain visible as a compact status notice.
- The previous-trick popover displays card faces with player captions and the
  winner. The winning card has a contrasting outline and an accessible label;
  an unfinished trick updates its highlight using led-color and trump rules.
  A localized status strip, highlighted like `PlayerBar`, identifies the viewer's
  actual card-playing turn (`next_player`), not eligibility to communicate.
  Opponents show a shared `Avatar` beside their info button, which takes the
  same highlight while they play next. The winning trick card uses the shared
  `highlighted` card state.
