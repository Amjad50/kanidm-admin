# 65 — OAuth2 Apps: Scope Maps Tab

The Scope Maps tab on the OAuth2 detail page. Manages standard and supplementary scope maps: which groups grant which OAuth2 scopes to their members.

## Purpose

Map kanidm groups to OAuth2 scope strings. When a user in a mapped group authenticates, kanidm includes the corresponding scopes in their token. Supplementary scope maps add further scopes on top of standard ones.

## Layout

Tab content inside the OAuth2 detail page. Two sections side-by-side or stacked: Standard scope maps + Supplementary scope maps.

```
┌─────────────────────────────────────────────────────────────────────┐
│ Scope maps                                                          │
│                                                                     │
│ ┌─────────────────────────────────────────────────────────────────┐ │
│ │ Standard scope maps                                             │ │
│ │ Groups in the standard map grant the listed scopes to their     │ │
│ │ members.                                                        │ │
│ │                                                                 │ │
│ │ ┌─────────────────────────────────────────────────────────────┐ │ │
│ │ │ Group         │ Scopes                            │ Actions │ │ │
│ │ │───────────────┼───────────────────────────────────┼─────────│ │ │
│ │ │ idm_admins    │ openid, profile, email, groups    │  Edit ✕│ │ │
│ │ │ developers    │ openid, profile, email, groups    │  Edit ✕│ │ │
│ │ │ vpn_users     │ openid, email                     │  Edit ✕│ │ │
│ │ └─────────────────────────────────────────────────────────────┘ │ │
│ │                                                                 │ │
│ │ [+ Add scope map]                                               │ │
│ └─────────────────────────────────────────────────────────────────┘ │
│                                                                     │
│ ┌─────────────────────────────────────────────────────────────────┐ │
│ │ Supplementary scope maps                                        │ │
│ │ Custom scopes layered on top of the standard ones, often for    │ │
│ │ app-specific permissions.                                       │ │
│ │                                                                 │ │
│ │ ┌─────────────────────────────────────────────────────────────┐ │ │
│ │ │ Group         │ Scopes                            │ Actions │ │ │
│ │ │───────────────┼───────────────────────────────────┼─────────│ │ │
│ │ │ idm_admins    │ grafana_admin                     │  Edit ✕│ │ │
│ │ └─────────────────────────────────────────────────────────────┘ │ │
│ │                                                                 │ │
│ │ [+ Add supplementary scope map]                                 │ │
│ └─────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────┘
```

## API data shape — critical notes

See `../api-reality.md`. The entry's `oauth2_rs_scope_map` is an array of **pre-formatted strings**, one per scope-map entry:

```
"oauth2-proxy-users@idm.home.amsh.dev: {\"email\", \"groups\", \"openid\"}"
```

Format: `{group_spn}: {Rust-HashSet-Debug-output}`. Order of scopes within the set is NOT stable (HashSet). The UI parser must split on the first `: ` and parse `{"a", "b", "c"}` as a comma+space delimited quoted-string list.

There may or may not be a separate `oauth2_rs_sup_scope_map` attr for supplementary scope maps (not directly observed in the surveyed instance; the CLI exposes `update-sup-scope-map` so the attr must exist when used).

**Crucial:** the UI must NEVER edit the scope map by mutating these strings. All edits go through the dedicated REST endpoints:
- `POST /v1/oauth2/{name}/_scopemap/{group}` (create / update)
- `DELETE /v1/oauth2/{name}/_scopemap/{group}` (delete)
- `POST /v1/oauth2/{name}/_sup_scopemap/{group}` (supplementary)
- `DELETE /v1/oauth2/{name}/_sup_scopemap/{group}`

After a mutation, refetch the entry to get the updated map.

## Tab content

### Standard scope maps section

**Header:** "Standard scope maps" + description "Groups in the standard map grant the listed scopes to their members."

**Table columns:**
- Group — group name (clickable, navigates to group detail)
- Scopes — comma-separated list of scope strings, monospace
- Actions — Edit + Delete (✕)

Edit opens an inline modal/popover with the scope editor (see "Scope map editor" below).

Delete: small confirm "Remove the scope map for group {name}? Members of this group will no longer receive these scopes from this application."

**Add button:** "+ Add scope map" — opens the scope map editor with empty fields.

If empty: "No standard scope maps. → Add a scope map to grant scopes to a group's members."

### Supplementary scope maps section

Same UX as standard, but for the supplementary attribute (kanidm distinguishes these as `_sup_scopemap` endpoints).

The visual treatment can be slightly different (e.g., a subtle accent color on the section header or a small "Supplementary" badge on each row) to make it clear these are additional, not standard.

If empty: "No supplementary scope maps."

## Scope map editor (add / edit)

Opens as a modal or inline popover:

```
   ┌──────────────────────────────────────────────────┐
   │  Edit scope map — developers              [×]    │
   ├──────────────────────────────────────────────────┤
   │                                                  │
   │  Group *                                         │
   │  ┌────────────────────────────────────────────┐  │
   │  │ developers                          [Change]│ │
   │  └────────────────────────────────────────────┘  │
   │                                                  │
   │  Scopes *                                        │
   │  ┌────────────────────────────────────────────┐  │
   │  │ ☑ openid                                   │  │
   │  │ ☑ profile                                  │  │
   │  │ ☑ email                                    │  │
   │  │ ☑ groups                                   │  │
   │  │ ☐ groups_uuid                              │  │
   │  │ ☐ groups_name                              │  │
   │  │ ☐ groups_spn                               │  │
   │  │ ☐ ssh_publickeys                           │  │
   │  │ ☐ read                                     │  │
   │  │ ☐ supplement                               │  │
   │  └────────────────────────────────────────────┘  │
   │                                                  │
   │  Custom scopes                                   │
   │  ┌────────────────────────────────────────────┐  │
   │  │                                            │  │
   │  └────────────────────────────────────────────┘  │
   │  Comma-separated. Custom scopes are passed       │
   │  through as-is to the token.                     │
   │                                                  │
   ├──────────────────────────────────────────────────┤
   │              [Cancel]       [Save scope map]     │
   └──────────────────────────────────────────────────┘
```

### Editor content

**Group field:**
- For Add: group picker (typeahead, same as elsewhere)
- For Edit: group name displayed read-only OR with "Change" button. Changing the group basically becomes "delete + add new" — UI should warn: "Changing the group will remove this map and create a new one for the selected group."

**Scopes field:**
- Multi-select checkbox list of standard kanidm scopes:
  - `openid` (OIDC required)
  - `profile`
  - `email`
  - `groups`
  - `groups_uuid` (group membership by UUID)
  - `groups_name`
  - `groups_spn`
  - `ssh_publickeys`
  - `read`
  - `supplement`
- Each option has a small tooltip / help icon explaining what it includes

**Custom scopes field:**
- Optional comma-separated text input for non-standard scopes (used for app-specific permissions like `grafana_admin`)
- Helper: "Comma-separated. Custom scopes are passed through as-is to the token."
- For the supplementary scope map editor, custom scopes are MORE prominent (custom is the typical use case for supplementary)

**Footer:** Cancel + Save scope map (disabled until at least one scope is selected — kanidm doesn't allow empty maps).

On save: POST/PUT to the appropriate scope map endpoint. Refresh the table.

## States

- **Loading:** skeleton tables.
- **Empty (no scope maps):** "No standard scope maps." section with prominent add button.
- **Adding/editing:** modal/popover state.
- **Saving:** Save button spinner.
- **Error:** toast or inline.

## Sample data

For `grafana` from `_sample-data.md`:

Standard scope maps:
- `idm_admins` → `openid, profile, email, groups`
- `developers` → `openid, profile, email, groups`
- `vpn_users` → `openid, email`

Supplementary scope maps:
- `idm_admins` → `grafana_admin`

For an empty-state mockup, use a freshly-created app (no maps yet).

## Edge cases

- **Group deleted while map exists:** the scope map entry shows the group name in `--danger` color with a "Group missing — remove this map" subdued line. Action: Delete.
- **Same group in both standard and supplementary:** allowed. Scopes are merged.
- **Custom scope conflicts with reserved scope:** kanidm rejects. Show inline error.
- **Saving with no scopes selected:** Save disabled.
- **Group not found in picker:** show "No groups match" — can't save with non-existent group.

## Mockup elements to render

- Tab content with "Scope maps" heading
- Standard scope maps section with table (3 rows for Grafana sample)
- Supplementary scope maps section with table (1 row: idm_admins → grafana_admin)
- "+ Add scope map" button below each table
- Render the scope map editor modal as a separate mockup (editing developers row): group field, scopes checkboxes with openid, profile, email, groups checked; custom scopes empty
- Render an empty-state mockup: both sections empty
- Render the supplementary editor variant where custom scopes are populated with `grafana_admin`
