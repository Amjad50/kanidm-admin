# 28 — People: Validity Tab

The Validity tab on the person detail page. Sets the account's validity window: when it becomes active (valid-from) and when it expires (expire-at).

## Purpose

Configure when a person can sign in. Supports onboarding (account active starting future date), offboarding (account expires on a specific date), and indefinite access (always valid / never expires). The kanidm CLI accepts RFC3339 datetimes and keyword shortcuts: `now`, `never`, `clear`, `any`, `epoch`. The UI should support all of these via a natural form.

## Layout

Tab content inside the person detail page:

```
┌─────────────────────────────────────────────────────────────────────┐
│ Validity                                                            │
│                                                                     │
│ Current status: ● Active                                            │
│ Account is valid from any time until forever.                       │
│                                                                     │
│ ┌─────────────────────────────────────────────────────────────────┐ │
│ │ Valid from                                                      │ │
│ │                                                                 │ │
│ │ ( ) Any time (default)                                          │ │
│ │ (•) Specific date                                               │ │
│ │     ┌───────────────────┐ ┌─────────────┐                       │ │
│ │     │ 2026-05-14        │ │ 09:00       │  Set to now           │ │
│ │     └───────────────────┘ └─────────────┘                       │ │
│ │                                                                 │ │
│ │     Time zone: UTC                                              │ │
│ └─────────────────────────────────────────────────────────────────┘ │
│                                                                     │
│ ┌─────────────────────────────────────────────────────────────────┐ │
│ │ Expires at                                                      │ │
│ │                                                                 │ │
│ │ (•) Never (default)                                             │ │
│ │ ( ) Specific date                                               │ │
│ │     ┌───────────────────┐ ┌─────────────┐                       │ │
│ │     │ 2026-12-31        │ │ 23:59       │  Set to now           │ │
│ │     └───────────────────┘ └─────────────┘                       │ │
│ │                                                                 │ │
│ │     Time zone: UTC                                              │ │
│ └─────────────────────────────────────────────────────────────────┘ │
│                                                                     │
│                                            [Discard] [Save validity]│
└─────────────────────────────────────────────────────────────────────┘
```

## Tab content sections

### Status banner

A short summary at the top showing the current effective validity:
- ● Active (green dot) when the account is currently valid
- ● Not yet active (warning dot) when valid-from is in the future
- ● Expired (danger dot) when expire-at is in the past

Below the dot+label, a single-sentence description:
- "Account is valid from any time until forever." (default — no dates set)
- "Account is valid from 2026-05-01 09:00 UTC until 2026-12-31 23:59 UTC."
- "Account expired on 2026-05-12. Sign-in is currently blocked."
- "Account is not active until 2026-06-01. Sign-in will be allowed after that."

### Valid from section

Card with:
- Section heading "Valid from"
- Radio choice:
  - **Any time (default)** — the account is valid from epoch (no lower bound). Maps to CLI keyword `any` or `clear`.
  - **Specific date** — when selected, shows two inputs: date picker + time picker (24h format). RFC3339 is constructed from these.
- Quick shortcut: "Set to now" link — fills the inputs with the current datetime (truncated to the next minute).
- Time zone hint below the inputs: "Time zone: UTC" (kanidm uses RFC3339 with timezone; this UI uses UTC for simplicity; designer's call whether to allow local-tz input with conversion display).

### Expires at section

Similar layout:
- Section heading "Expires at"
- Radio choice:
  - **Never (default)** — no upper bound. Maps to CLI keyword `never`.
  - **Specific date** — date + time pickers.
- Quick shortcut: "Set to now" (immediately disables the account — useful for emergency offboarding).
- Time zone hint.

### Footer

Right-aligned actions:
- "Discard" — reverts unsaved changes
- "Save validity" — primary; disabled until any change is made
- On save: there's NO dedicated validity endpoint. The UI calls per-attribute mutations:
  - Set valid-from: `PUT /v1/person/{id}/_attr/account_valid_from` body `["<RFC3339>"]`
  - Clear valid-from (= "any time"): `DELETE /v1/person/{id}/_attr/account_valid_from`
  - Set expire-at: `PUT /v1/person/{id}/_attr/account_expire` body `["<RFC3339>"]`
  - Clear expire-at (= "never"): `DELETE /v1/person/{id}/_attr/account_expire`
- The CLI keywords (`any`, `never`, `clear`, `now`) are translated client-side. `now` → current `OffsetDateTime` → RFC3339. `clear`, `any`, `never` → DELETE call.

## States

- **Idle:** as described, with current values pre-populated.
- **Modified:** Save button enabled; "Unsaved changes" indicator near footer.
- **Saving:** Save button shows spinner.
- **Saved:** toast "Validity updated" + status banner refreshes.
- **Error:** toast or inline message.

## Sample data

For Alice Smith (currently active, no dates set):
- Status: ● Active. "Account is valid from any time until forever."
- Valid from: Any time selected
- Expires at: Never selected

For Frank Future (valid-from in 2 weeks):
- Status: ● Not yet active (warning).
- "Account is not active until 2026-05-28 09:00 UTC."
- Valid from: Specific date — `2026-05-28` `09:00`
- Expires at: Never

For Dave Locked (expired):
- Status: ● Expired (danger).
- "Account expired on 2026-05-11 23:59 UTC. Sign-in is currently blocked."
- Valid from: Any time
- Expires at: Specific date — `2026-05-11` `23:59`

For an onboarding scenario:
- Valid from: `2026-06-01` `09:00`
- Expires at: `2026-12-31` `17:00`

## Edge cases

- **Valid-from is after expires-at:** show inline error: "Valid-from must be earlier than expires-at." Save disabled.
- **Setting expires-at to the past:** allowed (it immediately disables the account). Show a confirm dialog: "This will deactivate the account immediately. Continue?" with Cancel / Continue.
- **Setting valid-from to the past:** allowed; just means the account is active now.
- **Person currently signed in when validity expires:** kanidm handles session invalidation server-side. UI doesn't need to do anything special; the next API call from the signed-in person fails.
- **CLI keyword: `clear` vs. unset:** in kanidm, "clear" means remove the attribute. The UI maps "Any time" / "Never" to clearing the respective attribute (effectively unsetting it).

## Tone

Functional and clear. The Validity tab is sometimes the only thing standing between an admin and a properly-offboarded employee — make the danger states (expired, future-dated) visible and the actions safe.

## Mockup elements to render

- Tab content with "Validity" heading
- Status banner with green dot "Active" + "valid from any time until forever" description
- Valid from section with radio options, Any time selected
- Expires at section with radio options, Never selected
- Footer with Discard + Save validity buttons (Save disabled)
- Render a second variant: Frank Future scenario — valid-from filled with `2026-05-28 09:00`, status banner showing "Not yet active" in warning color
- Render a third variant: Dave Locked scenario — expires-at filled with `2026-05-11 23:59`, status banner showing "Expired" in danger color
