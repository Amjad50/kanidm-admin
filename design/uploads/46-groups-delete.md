# 46 — Groups: Delete Confirm Modal

Confirmation modal for deleting a group. Type-to-confirm pattern.

## Purpose

Make group deletion intentional. Show consequences (member removals, scope-map invalidations, etc.). Block deletion of built-in groups. Require typed confirmation.

## Layout

Modal overlay using the design system's modal spec (~520-560px wide).

```
   ┌──────────────────────────────────────────────────┐
   │  Delete group                             [×]    │
   ├──────────────────────────────────────────────────┤
   │                                                  │
   │  ⚠ You're about to delete:                       │
   │                                                  │
   │  👥  developers                                   │
   │     24 members                                   │
   │                                                  │
   │  What happens:                                   │
   │   ▸ The group is moved to the recycle bin and    │
   │     is recoverable for 7 days.                   │
   │   ▸ Its 24 members lose group membership         │
   │     immediately, including any associated        │
   │     account policy and OAuth2 scope maps.        │
   │   ▸ OAuth2 scope maps using this group will      │
   │     stop granting their scopes.                  │
   │   ▸ Any group using this as its                  │
   │     entry-managed-by will fall back to the       │
   │     default (idm_admins).                        │
   │                                                  │
   │  Type the group name to confirm:                 │
   │  developers                                      │
   │  ┌────────────────────────────────────────────┐  │
   │  │                                            │  │
   │  └────────────────────────────────────────────┘  │
   │                                                  │
   ├──────────────────────────────────────────────────┤
   │              [Cancel]       [Delete group]       │
   └──────────────────────────────────────────────────┘
```

## Modal content

**Header:** "Delete group" + close button.

**Body:**

*Lead-in:* "⚠ You're about to delete:" with danger icon.

*Identity card:*
- Group icon (Lucide `UsersRound`) in soft-background container
- Group name (large, primary): "developers"
- Member count subtitle: "24 members"

*"What happens" section:* bulleted list:
- ▸ "The group is moved to the recycle bin and is recoverable for 7 days."
- ▸ "Its 24 members lose group membership immediately, including any associated account policy and OAuth2 scope maps." (only if N > 0)
- ▸ "OAuth2 scope maps using this group will stop granting their scopes."
- ▸ "Any group using this as its entry-managed-by will fall back to the default (idm_admins)."

The text adapts to the actual group state:
- If group has 0 members, omit the "members lose membership" bullet
- If no OAuth2 scope maps reference this group, omit the scope-map bullet
- If no group has this as entry-managed-by, omit the entry-managed-by bullet

*Type-to-confirm input:*
- "Type the group name to confirm:"
- Group name displayed in monospace, copyable
- Input field below — must match exactly to enable Delete

**Footer:** Cancel + Delete group (danger primary, disabled until input matches).

## Built-in group prevention

If the admin tries to delete a built-in group (`idm_admins`, `idm_*` system groups, etc.), the modal does NOT show the type-to-confirm input. Instead it shows:

```
   ┌──────────────────────────────────────────────────┐
   │  Cannot delete                             [×]   │
   ├──────────────────────────────────────────────────┤
   │                                                  │
   │  ✋ idm_admins is a built-in group and cannot    │
   │  be deleted.                                     │
   │                                                  │
   │  Built-in groups are part of kanidm's required   │
   │  configuration. To remove privileges from        │
   │  someone, edit the group's members instead.      │
   │                                                  │
   ├──────────────────────────────────────────────────┤
   │                  [Close]                         │
   └──────────────────────────────────────────────────┘
```

A single Close button. Built-in groups should also have their kebab "Delete" menu item disabled with the same tooltip in screens 40 and 42.

## States

- **Idle:** Delete disabled, input empty.
- **Match:** Delete enabled.
- **Deleting:** spinner.
- **Success:** modal closes, toast "Group deleted: developers — recoverable from the recycle bin for 7 days." Navigate to `/groups` or remove row from list.
- **Error:** inline at top, "Could not delete: {server message}". Cancel re-enabled.

## Sample data

Use `developers`:
- Group name: `developers`
- Members: 24
- Consequence list reflects 24 members, scope maps from `grafana` (which uses `developers` for scope), no entry-managed-by dependencies

For the built-in prevention variant, use `idm_admins`.

## Edge cases

- **Privilege required:** opens reauth modal (08) on Delete click.
- **Group has OAuth2 scope-map dependencies:** the consequence bullet adapts to count them: "OAuth2 scope maps in 3 applications use this group and will stop granting their scopes."
- **Bulk delete (from groups list):** similar to bulk delete in people — list of group names + type `DELETE` to confirm.

## Mockup elements to render

- Modal with backdrop blur
- Header "Delete group" + close
- Body with warning icon + lead-in
- Identity card for `developers` with member count
- "What happens" bulleted list
- Type-to-confirm with group name in monospace + copy button
- Empty input field below
- Footer with Cancel + Delete group (disabled)
- Render a second variant with input filled `developers` and Delete enabled
- Render the built-in prevention variant for `idm_admins`
