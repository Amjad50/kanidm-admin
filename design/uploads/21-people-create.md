# 21 — People: Create New Person

A simple form to create a new person. Kanidm's `POST /v1/person` only accepts `name` and `displayname`; all other attributes are set via update after creation.

## Purpose

Add a new person to the system. Capture the minimum required fields. After creation, redirect to the new person's detail page so the admin can add email, validity, credentials, etc.

## Layout

Inside the app shell. Main content area:

```
┌─────────────────────────────────────────────────────────────────────┐
│ People > Create person                                              │
│                                                                     │
│ Create a new person                                                 │
│ Just the basics — you can add email, set credentials, and configure │
│ groups on the next page.                                            │
│                                                                     │
│ ┌─────────────────────────────────────────────────────────────────┐ │
│ │ Username *                                                      │ │
│ │ ┌───────────────────────────────┐ @idm.example.com              │ │
│ │ │ jane.doe                      │                               │ │
│ │ └───────────────────────────────┘                               │ │
│ │ Used as the login name and in the SPN. Lowercase letters,       │ │
│ │ numbers, dot, underscore, hyphen. Cannot be changed after        │ │
│ │ creation without consequences.                                  │ │
│ │                                                                 │ │
│ │ Display name *                                                  │ │
│ │ ┌───────────────────────────────────────────────────────────┐   │ │
│ │ │ Jane Doe                                                  │   │ │
│ │ └───────────────────────────────────────────────────────────┘   │ │
│ │ Shown in lists and on the person's profile.                     │ │
│ │                                                                 │ │
│ │              [Cancel]            [Create person]                │ │
│ └─────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────┘
```

## Page header

- Breadcrumb: `People > Create person`
- Page title: "Create a new person"
- Subtitle: "Just the basics — you can add email, set credentials, and configure groups on the next page."

## Form

A single card with two required inputs.

### Username field

- Label: "Username *"
- Input type: text, autofocus
- Right-aligned suffix shown adjacent (not inside) the input: `@idm.example.com` in monospace, subdued text — making it obvious the SPN will be `{username}@idm.example.com`
- Placeholder: `jane.doe`
- Allowed characters: lowercase letters, digits, `.`, `_`, `-` (kanidm constraints)
- Inline validation:
  - As-you-type: invalid characters get a `--danger` border + inline error "Use lowercase letters, numbers, dot, underscore, or hyphen."
  - On blur with empty: "Username is required."
- Helper text below: "Used as the login name and in the SPN. Lowercase letters, numbers, dot, underscore, hyphen. Cannot be changed after creation without consequences."

### Display name field

- Label: "Display name *"
- Input type: text
- Placeholder: `Jane Doe`
- Helper text: "Shown in lists and on the person's profile."
- On blur with empty: "Display name is required."

### Footer actions

- Cancel — secondary button. Navigates back to `/people` (the list).
- Create person — primary button. Disabled until both fields are valid. On click:
  - Validates client-side
  - Calls `POST /v1/person` with `{name, displayname}`
  - On 201: shows success toast "Person created: jane.doe@idm.example.com" and navigates to `/people/jane.doe` (the new person's detail page, where the admin can continue setting up)
  - On 409 (already exists): inline error on username field "A person with this username already exists." Button returns to idle.
  - On 422 (validation): inline error on the offending field per server response.
  - On network error: toast "Could not create person. Try again."

## States

- **Idle:** as described.
- **Submitting:** Create button shows spinner + "Creating…" label. Inputs are read-only. Cancel still works.
- **Success:** form briefly shows a success state (button checkmark), then page navigates.
- **Conflict (duplicate name):** see above.
- **Network error:** see above.

## Sample data

Throughout this brief and the mockup, use a NEW sample person not in the existing list (since this is "create"):
- Username: `jane.doe`
- Display name: `Jane Doe`
- SPN it will become: `jane.doe@idm.example.com`

## Edge cases

- **Browser autofill:** if the browser tries to fill these fields, accept it but rely on user submitting.
- **Reserved usernames:** kanidm may reject some reserved names (e.g., starting with underscore). Show the server's error text inline if rejected.
- **Name collision with deleted person:** if a person was deleted but is still in the recycle bin, kanidm may reject the create. Error message: "A deleted person uses this username. Restore or purge that account first." (Recycle bin is out of scope for this admin UI; mention the existence but don't link.)
- **Pasted display name with newlines:** strip newlines before submitting.

## Tone

Helpful and minimal. The form has only two fields; don't overload with copy. The post-creation message ("you can add email, etc. on the next page") sets expectations cleanly.

## Mockup elements to render

- Breadcrumb "People > Create person"
- Title "Create a new person" + subtitle
- Form card with:
  - Username field showing `jane.doe` typed, with `@idm.example.com` suffix
  - Display name field showing `Jane Doe` typed
  - Helper text under each field
- Footer with Cancel + Create person buttons (Create is primary, enabled)
- Render a second state showing a duplicate-username error: username field has danger border, error message below: "A person with this username already exists."
