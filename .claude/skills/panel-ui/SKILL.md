---
name: panel-ui
description: How panel components are laid out and how they handle loading, actions, errors and busy states. Load when adding or editing src/App.tsx, anything under src/features/, or src/styles.css.
---

The panel has no tests, so follow the existing components exactly: `App.tsx`, `features/connect/ConnectForm.tsx`, `features/status/ConnectionRow.tsx`, `features/status/UsageLine.tsx`.

Layout

- A component lives in `src/features/<area>/<Name>.tsx` as a named `export function <Name>({ ... }: { ... })`, props typed inline. Only `App.tsx` sits at the root.
- Reach the command only through the wrappers in `src/lib/cli.ts`. Import result types with `import type { ... } from "../../lib/schema"`. Never call `invoke` from a component.

State

- Data loaded asynchronously is a union discriminated on `phase`: `{ phase: "loading" } | { phase: "ready"; ... } | { phase: "failed"; message: string }`, as in `App.tsx` and `UsageLine.tsx`.
- A multi-step flow extends the union with its own phases. `ConnectForm`'s `Progress` uses `idle`, `starting`, `waiting`, `finishing`, `failed`.

Actions

Two shapes. Pick the one that matches what the action does.

- One-shot: a busy flag and a failure string in their own state. An `async function` sets the flag, clears the previous failure, then `try { await <wrapper>(...) } catch (error) { failure = String(error) } finally { clear busy }`. See `act` in `ConnectionRow` (`busy`, `failure`) and `sync` in `App` (`syncing`, `notice`).
- Multi-step: driven from the `phase` union. `submit` in `ConnectForm` sets `{ phase: "starting" }` (which replaces an earlier `failed`), derives `busy` from `starting`/`waiting`/`finishing`, moves the phase on from the wrapper's events, sets `idle` on success and `failed` with `String(error)` in `catch`. No separate busy flag, no failure clearing, no `finally`.
- After a change succeeds, call the parent's callback (`onChanged`, `onConnected`). The parent re-runs `status`. Do not patch the status locally.
- In `App.tsx`, `sync` and `change` send errors to `notice`, rendered as `<p className="hint">`, and call `refresh()` whatever the outcome. `change`, wrapping the two footer toggles, has no busy flag: it awaits the wrapper, stores any error in `notice` and refreshes. The refresh's `loading` phase is the only sign a toggle is being applied.

Rendering

- Show a failure as `className="problem problem-broken"` holding the message text. Exception: `UsageLine`'s `failed` phase renders `<div className="connection-detail" title={state.message}>Usage unavailable</div>`, fixed text with the message in `title`, because the usage line is supplementary and must not look like a fault in the connection. Use this only for a secondary line whose failure does not affect what the user can do.
- Health-coloured elements use the health value in the class: `dot-${health}`, `problem-${health}`.
- Buttons are `type="button"`, except the form's submit button.
- A button that runs an action is `disabled` while its own component's action is in flight: `busy !== null` in `ConnectionRow`, `syncing` for Sync now and `state.phase === "loading"` for Check again in `App`, the derived `busy` in `ConnectForm`. The two footer checkboxes in `App.tsx` are never disabled.
- A busy button's label is the present participle plus `…`: "Syncing…", "Checking…", "Repairing…", "Disconnecting…", "Connecting…".
- A button that opens a confirmation or form ends in `…`: "Disconnect…", "Connect another Modelwise…". A destructive confirm button gets `className="danger"`.
- Secondary text uses `hint` or `connection-detail`.

Styles

- All styles are in `src/styles.css`. Colours come from the custom properties in `:root`, which have dark-mode values. Never write a literal colour in a rule.
