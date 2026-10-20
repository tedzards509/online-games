# Frontend design

## Direction

A quiet, friendly card table: pale green canvas, cream surfaces, deep green
controls and readable rounded type. Use warm red sparingly for focus, errors,
and red suits. Keep the interface practical rather than decorative.

- Show only what is needed to join, create, or play. The landing page needs no
  branding, headline, explanatory copy, or recent-room list.
- Place the nickname above separate create/join panels. Use two columns on
  desktop and tablet, and one on narrow screens. Invitation links show joining
  only, with the room code filled in.
- Keep settings in the upper-right on larger screens and lower-right on mobile.
  The language picker is a small dismissible popover, not a separate page.
- Use comfortable touch targets, visible keyboard focus, and placeholder hints
  backed by accessible field names. Never require hover to operate a control.
- Avoid ornamental motion, tilted buttons, decorative tables, or hand layouts
  until a particular game needs them.

## Implementation sources

Exact values and their roles live in [`frontend/src/design.css`](../frontend/src/design.css).
Component-specific layout lives alongside the Svelte markup, not in this document.
See [`frontend/README.md`](../frontend/README.md) for component contracts and examples.

Nunito Sans is bundled with the application. Fonts and translations require no
external requests. English, German, and German (cool) have independent catalogs.
Use the first supported device language unless the user has saved a selection;
English is the fallback. German (cool) is an explicit selection, not a device locale.

## Playing cards

All cards share a rounded 5:7 shell. Layer symbol/letter cards onto that shell,
then standard playing cards onto the symbol layer. Face-down cards use the shell
without receiving hidden face data. Standard suits use SVG, keeping their rendering
independent of installed fonts.

Cards can be passive displays or native buttons. Accessible names describe their
visible meaning; a selectable button can expose its selected state. Games own
their hand/table layouts, legal-action behavior, and responsive card sizing.

## Scope

The shared components, landing flow, and language settings use this design.
Room setup and individual game boards retain their own presentation and English
copy until explicitly included in a design pass. Do not apply a new theme to
existing boards merely because a shared component is available.
