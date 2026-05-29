# 41 — Groups: Create New Group

A simple form to create a new group. Per kanidm CLI: name (required) + optional entry-managed-by.

## Purpose

Create a new group. Capture name + optional entry-managed-by. After creation, redirect to the new group's detail page so the admin can add members, description, mail, account policy, etc.

## Layout

Inside the app shell:

```
┌─────────────────────────────────────────────────────────────────────┐
│ Groups > Create group                                               │
│                                                                     │
│ Create a new group                                                  │
│ Add members, mail, and account policy on the next page.             │
│                                                                     │
│ ┌─────────────────────────────────────────────────────────────────┐ │
│ │ Group name *                                                    │ │
│ │ ┌─────────────────────────────────────────────────────────────┐ │ │
│ │ │ frontend_devs                                               │ │ │
│ │ └─────────────────────────────────────────────────────────────┘ │ │
│ │ Lowercase letters, numbers, underscore. The group's SPN will   │ │
│ │ be frontend_devs@idm.example.com.                              │ │
│ │                                                                 │ │
│ │ Entry managed by                                               │ │
│ │ ┌─────────────────────────────────────────────────────────────┐ │ │
│ │ │ 🔍 Search groups…                                            │ │ │
│ │ └─────────────────────────────────────────────────────────────┘ │ │
│ │ Optional. The group that can manage this group's attributes    │ │
│ │ and membership. Defaults to idm_admins.                        │ │
│ │                                                                 │ │
│ │            [Cancel]            [Create group]                   │ │
│ └─────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────┘
```

## Page header

- Breadcrumb: `Groups > Create group`
- Title: "Create a new group"
- Subtitle: "Add members, mail, and account policy on the next page."

## Form

### Group name field

- Label "Group name *"
- Required
- Validation: lowercase letters, digits, underscore (match kanidm's group name constraints — same charset as person names)
- Placeholder: `frontend_devs`
- Helper text: "Lowercase letters, numbers, underscore. The group's SPN will be `{name}@idm.example.com`."
- Inline validation as-you-type for invalid chars

### Entry managed by field

- Label "Entry managed by"
- A searchable group picker (typeahead input)
- Optional — empty defaults to `idm_admins` per kanidm convention
- Helper: "Optional. The group that can manage this group's attributes and membership. Defaults to idm_admins."
- Behavior: as the admin types, query `GET /v1/group?...` and show matching groups as a dropdown
- Clear button (×) when value is selected

### Footer

- Cancel — navigate to `/groups`
- Create group — primary, disabled until name is valid
- On success: navigate to `/groups/{name}` with toast "Group created: frontend_devs"
- On 409 (duplicate name): inline error "A group with this name already exists."

## States

- **Idle:** as described.
- **Submitting:** Create button spinner + "Creating…"; inputs read-only.
- **Conflict:** inline error.
- **Network error:** toast.

## Sample data

For the mockup:
- Group name: `frontend_devs` (NEW name — not in the existing sample groups list)
- Entry managed by: empty (defaults to `idm_admins`)

For the entry-managed-by picker dropdown (when active), show the existing sample groups as options:
- `idm_admins`, `developers`, `devops`, `system_admins`

## Edge cases

- **Name starts with `idm_` prefix:** kanidm may reserve this prefix for system groups. If server rejects, show inline error verbatim.
- **Whitespace in name:** strip leading/trailing; reject inner whitespace inline.
- **Reserved names (e.g., `anonymous`):** server rejects; show inline error.
- **Entry-managed-by typo:** if the admin types a non-existent group name, the picker shows "No groups match" — they can't submit a non-existent value (picker only allows selecting from results).

## Mockup elements to render

- Breadcrumb
- Title + subtitle
- Form card:
  - Group name field with `frontend_devs` typed
  - Entry managed by field with search placeholder and no value selected
- Cancel + Create group buttons
- Render a second variant showing the entry-managed-by picker open with the sample group list as dropdown options
