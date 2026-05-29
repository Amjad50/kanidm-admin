# 66 — OAuth2 Apps: Claim Maps Tab

The Claim Maps tab on the OAuth2 detail page. Manages custom JWT claims: which groups inject which arbitrary values into a named claim, with a join strategy controlling how multiple values are merged.

## Purpose

Map kanidm groups to custom claim values. For each named claim, multiple group → values entries can exist; kanidm merges them per the join strategy (csv / ssv / array). The CLI exposes `update-claim-map`, `delete-claim-map`, and `update-claim-map-join`.

## Layout

Tab content inside the OAuth2 detail page:

```
┌─────────────────────────────────────────────────────────────────────┐
│ Claim maps                                                          │
│                                                                     │
│ Custom claims injected into the user's token, mapped from group     │
│ membership. Each claim has a join strategy controlling how multiple │
│ group values combine.                                               │
│                                                                     │
│ ┌─────────────────────────────────────────────────────────────────┐ │
│ │ nextcloud_quota                                  Join: array    │ │
│ │ ┌─────────────────────────────────────────────────────────────┐ │ │
│ │ │ Group         │ Values                          │ Actions   │ │ │
│ │ │───────────────┼─────────────────────────────────┼───────────│ │ │
│ │ │ developers    │ 50GB                            │ Edit ✕    │ │ │
│ │ │ idm_admins    │ unlimited                       │ Edit ✕    │ │ │
│ │ └─────────────────────────────────────────────────────────────┘ │ │
│ │ [+ Add group to nextcloud_quota]                                │ │
│ │ [Change join strategy] [Delete claim]                           │ │
│ └─────────────────────────────────────────────────────────────────┘ │
│                                                                     │
│ ┌─────────────────────────────────────────────────────────────────┐ │
│ │ department                                       Join: csv      │ │
│ │ ┌─────────────────────────────────────────────────────────────┐ │ │
│ │ │ Group         │ Values                          │ Actions   │ │ │
│ │ │───────────────┼─────────────────────────────────┼───────────│ │ │
│ │ │ developers    │ Engineering, Product            │ Edit ✕    │ │ │
│ │ │ devops        │ Engineering, Infrastructure     │ Edit ✕    │ │ │
│ │ └─────────────────────────────────────────────────────────────┘ │ │
│ │ [+ Add group to department]                                     │ │
│ │ [Change join strategy] [Delete claim]                           │ │
│ └─────────────────────────────────────────────────────────────────┘ │
│                                                                     │
│ [+ Add new claim]                                                   │
└─────────────────────────────────────────────────────────────────────┘
```

## API data shape — critical notes

See `../api-reality.md`. The entry's `oauth2_rs_claim_map` is an array of **pre-formatted colon-delimited strings**, one per claim+group entry:

```
"policy:minio-admins@idm.home.amsh.dev:,:\"allAccess\""
```

Format: `{claim_name}:{group_spn}:{join_char}:{value(s)}` — colon-separated. The `join_char` is the actual character used: `,` for csv, ` ` (space) for ssv, possibly `;` or some marker for array.

Multi-value claims may encode their values as `"v1","v2","v3"` in the value position — needs verification before implementing.

The UI parser is brittle here. Recommendation: parse defensively and gracefully degrade to displaying the raw string with an "(could not parse)" warning if parsing fails for an unfamiliar shape. Never overwrite the field via PUT — only use the dedicated endpoints:
- `POST /v1/oauth2/{name}/_claimmap/{claim}/{group}` (create / update)
- `DELETE /v1/oauth2/{name}/_claimmap/{claim}/{group}`
- `POST /v1/oauth2/{name}/_claimmap/{claim}` (set join strategy)

After mutation, refetch.

## Tab content

### Page-level description

Above all claim cards:
"Custom claims injected into the user's token, mapped from group membership. Each claim has a join strategy controlling how multiple group values combine."

### Claim card (one per named claim)

Each claim has its own card containing:

**Header:**
- Left: claim name (monospace, primary)
- Right: "Join: {strategy}" badge — clickable to change

**Table:**
- Group (clickable to group detail)
- Values (comma-separated values for this group)
- Actions: Edit + Delete (✕)

**Bottom actions:**
- "+ Add group to {claim_name}" — opens entry editor for this claim
- "Change join strategy" — opens a small popover to switch csv / ssv / array
- "Delete claim" — danger; deletes the entire claim mapping (all group entries)

### Add new claim

Bottom of the tab: "+ Add new claim" — opens a flow to:
1. Name the claim
2. Pick a join strategy
3. Add first group + values

This flow can be a multi-step modal or a single form. Designer's call.

### Claim map editor (add / edit a group-values entry)

Opens as modal/popover:

```
   ┌──────────────────────────────────────────────────┐
   │  Edit claim map — nextcloud_quota (developers)   │
   │                                            [×]   │
   ├──────────────────────────────────────────────────┤
   │                                                  │
   │  Claim name                                      │
   │  nextcloud_quota  (read-only for edit; editable  │
   │  for new claim)                                  │
   │                                                  │
   │  Group *                                         │
   │  ┌────────────────────────────────────────────┐  │
   │  │ developers                          [Change]│ │
   │  └────────────────────────────────────────────┘  │
   │                                                  │
   │  Values *                                        │
   │  ┌────────────────────────────────────────────┐  │
   │  │ 50GB                                       │  │
   │  └────────────────────────────────────────────┘  │
   │  [+ Add another value]                           │
   │  One value per line, or use the join strategy   │
   │  configured for this claim.                     │
   │                                                  │
   │  Join strategy (for this claim)                  │
   │  ( ) csv — comma-separated string                │
   │  ( ) ssv — space-separated string                │
   │  (•) array — JSON array                          │
   │  Shown for reference; change via "Change join    │
   │  strategy" on the claim card.                    │
   │                                                  │
   ├──────────────────────────────────────────────────┤
   │              [Cancel]       [Save claim map]     │
   └──────────────────────────────────────────────────┘
```

### Change join strategy popover

Small popover anchored to the badge or "Change join strategy" link:

```
   ┌────────────────────────────────────┐
   │  Join strategy for                 │
   │  nextcloud_quota                   │
   │                                    │
   │  ( ) csv  — "50GB,unlimited"       │
   │  ( ) ssv  — "50GB unlimited"       │
   │  (•) array — ["50GB","unlimited"]  │
   │                                    │
   │       [Cancel]     [Save]          │
   └────────────────────────────────────┘
```

Each radio shows a small example with the current group values to make the difference concrete. Saving calls the `_claimmap/{claim_name}` join endpoint.

## Add new claim flow

Triggered by "+ Add new claim". A modal:

```
   ┌──────────────────────────────────────────────────┐
   │  Add new claim                            [×]    │
   ├──────────────────────────────────────────────────┤
   │                                                  │
   │  Claim name *                                    │
   │  ┌────────────────────────────────────────────┐  │
   │  │ team                                       │  │
   │  └────────────────────────────────────────────┘  │
   │  The name of the JWT claim. Use lowercase with   │
   │  underscores.                                    │
   │                                                  │
   │  Join strategy *                                 │
   │  ( ) csv      ( ) ssv      (•) array             │
   │                                                  │
   │  First entry: Group *                            │
   │  ┌────────────────────────────────────────────┐  │
   │  │ 🔍 Search groups…                          │  │
   │  └────────────────────────────────────────────┘  │
   │                                                  │
   │  Values *                                        │
   │  ┌────────────────────────────────────────────┐  │
   │  │ backend                                    │  │
   │  └────────────────────────────────────────────┘  │
   │  [+ Add another value]                           │
   │                                                  │
   ├──────────────────────────────────────────────────┤
   │              [Cancel]       [Create claim]       │
   └──────────────────────────────────────────────────┘
```

## States

- **Loading:** skeleton claim cards.
- **Empty (no claim maps):** "No claim maps configured." + "+ Add new claim" prominent button.
- **Editing entry / changing join / adding new claim:** modal active.
- **Saving:** spinner.
- **Error:** inline or toast.

## Sample data

For `nextcloud` from `_sample-data.md` (which has claim maps):

Claim: `nextcloud_quota`, join: array
- `developers` → `50GB`
- `idm_admins` → `unlimited`

Claim: `department`, join: csv
- `developers` → `Engineering`, `Product`
- `devops` → `Engineering`, `Infrastructure`

For Grafana (no claim maps): empty state.

## Edge cases

- **Claim with 0 group entries:** kanidm probably doesn't allow this; deleting the last entry deletes the claim. Show confirm: "Removing the last group entry will delete the {claim_name} claim. Continue?"
- **Renaming a claim:** kanidm doesn't directly support — it's delete + add. UI can disable rename or provide a "Rename claim" flow that does delete-then-create.
- **Join strategy mismatch UI:** if values for a group entry are typed as comma-separated text but the join strategy is array, kanidm parses them as separate array elements. UI should clarify: each value is a separate entry; the join controls how they're emitted in the token. The editor's "+ Add another value" pattern with one value per row makes this clearer than a CSV input field.
- **Very long values:** truncate in the table with full value on hover.

## Mockup elements to render

- Tab content with "Claim maps" heading + description
- Two claim cards: `nextcloud_quota` (join: array, 2 group entries) and `department` (join: csv, 2 group entries)
- All sample values populated
- "+ Add new claim" button at bottom
- Render the claim map editor modal as a second variant (editing the developers row for nextcloud_quota)
- Render the "Change join strategy" popover with the current values showing in the examples
- Render the empty state for Grafana (no claim maps) as a third variant
