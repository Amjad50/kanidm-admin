# 91 — Error States (Cross-Cutting Pattern)

Consistent treatment for all types of errors: inline field errors, page-level errors, HTTP errors (401 / 403 / 404 / 500), and toasts.

## Purpose

Every error tells the user what went wrong, why (if known and not a security leak), and what to do next. Errors should be honest, not cryptic, and never make the user feel they did something wrong unless they did.

## Error categories

### 1. Inline field error

For form validation failures (invalid input, required field empty, duplicate name, etc.).

**Appearance:**
- Input border in `--danger`
- Error message below the input, ~12-13px, `--danger` text, with optional small `AlertCircle` icon at the start
- The page does not scroll; the error appears below the input

**Examples:**
- "Username is required."
- "Use lowercase letters, numbers, dot, underscore, or hyphen."
- "A person with this username already exists."
- "Enter a full URL including https://"
- "Code must be 6 digits."

Inline errors clear as soon as the user starts editing the field again.

### 2. Form-level error (banner at top of form)

For server-side validation that spans multiple fields, or when an entire form submission failed.

**Appearance:**
- A banner at the top of the form card with `--danger-soft` background, `--danger` text, optional close button
- Icon: `AlertCircle` or similar
- Message + optional retry button

**Example:**
"Could not save the OAuth2 settings. Some fields conflict with the server's current configuration. Try refreshing the page and re-applying."

### 3. Page-level error

For pages that failed to load entirely (e.g., the detail page for an entity that errored out).

**Appearance:**
- Centered, similar to empty state, but with `--danger` semantic color in the icon
- Heading + body + "Retry" button

**Example:**
- Icon: `AlertOctagon`
- Heading: "Could not load this page"
- Body: "Something went wrong fetching the data for grafana. Try again, or check that the kanidm server is reachable."
- Action: "Retry" button

### 4. Toast error

For background or async failures (e.g., a save that failed, a network blip).

**Appearance:**
- Top-right toast (per design system's toast spec)
- Left border or icon in `--danger`
- Title + body + dismiss button
- Does NOT auto-dismiss (user must close manually — errors are important)

**Example:**
- Title: "Could not generate reset link"
- Body: "The server returned a 500 error. Try again or check the server logs."
- Action: "Retry" link OR "Dismiss" button

### 5. HTTP status pages (full-page)

For navigation to non-existent resources or for permission errors.

**404 — Not found:**
- Heading: "Not found"
- Body: "The page you're looking for doesn't exist. It may have been deleted, or the URL is wrong."
- Buttons: "Go back" (browser back), "Dashboard" (navigate home)

**403 — Forbidden:**
- Heading: "You don't have access"
- Body: "Your account doesn't have permission to view this. Contact an administrator if you think this is a mistake."
- Buttons: "Go back", "Dashboard"

**401 — Unauthenticated:**
- Generally handled by auto-redirect to login (screen 01) with intent preservation
- If displayed: "Your session expired. Sign in again to continue."
- Button: "Sign in"

**500 — Server error:**
- Heading: "Something went wrong"
- Body: "The kanidm server returned an unexpected error. Try again, or check the server logs."
- Body extra (small, subdued): "Error ID: {request_id}" — for support correlation
- Buttons: "Retry", "Go back"

**Network offline / unreachable:**
- Banner at top of main content area, persistent until reconnected
- Tone: warning, not danger (it's recoverable)
- "You're offline. Reconnecting…" with a small spinner

### 6. Validation errors with structured detail

For complex forms (account policy, scope maps with multiple validation failures at once), the server may return multiple errors. Display them as:
- Inline errors on each affected field
- Plus a summary banner at top: "3 errors to fix" with a list

## Common error messaging patterns

**Avoid:**
- "Oops! Something went wrong." (uninformative, infantilizing)
- "Sorry, we had a problem." (apologetic, vague)
- "Please try again later." (defeatist, no info)
- Raw stack traces or developer messages

**Prefer:**
- "Could not {action}: {specific reason}."
- "{Specific thing} is {specific problem}."
- "Try {next step}."

Examples:
- ✓ "Could not delete person: the account is referenced by an active OAuth2 scope map. Remove it from the scope map first."
- ✗ "Delete failed: BackendError(IntegrityViolation)"

## When to show what

| Failure source | Show as |
|---|---|
| Client-side validation | Inline field error |
| Server-side validation (one field) | Inline field error |
| Server-side validation (multiple fields) | Form-level banner + inline |
| Failed save / submit | Toast OR form-level banner |
| Failed background refresh | Toast (non-blocking) |
| Failed initial page load | Page-level error |
| 401 | Redirect to login |
| 403 | Page-level forbidden message OR toast (depends on context) |
| 404 | Page-level not-found |
| 500+ | Page-level error OR toast for non-blocking operations |
| Network offline | Top-of-page banner |

## Accessibility

- All error messages must have appropriate ARIA roles (`role="alert"` for important, `role="status"` for less urgent).
- Errors must be announced to screen readers when they appear.
- Color is never the only indicator — icons and text accompany the color.

## Reduce-motion considerations

- Toast slide-in animations respect reduce-motion (opacity-only).
- Banner appearance is instant if reduce-motion is set.

## Sample data references

- 404 example: navigating to `/people/nonexistent`
- 403 example: a person who lacks `idm_admins` membership trying to access OAuth2 management
- 500 example: a save fails server-side
- Validation example: creating a person with a duplicate username

## Mockup elements to render

Render 5 distinct error mockups:

1. **Inline field error** — a form with the username input in danger border, error message "A person with this username already exists." below
2. **Form-level banner** — a multi-field form with a banner at top: "3 errors to fix" + inline errors on each affected field
3. **Page-level error** — full main-content area with the "Could not load this page" centered pattern
4. **Toast error** — top-right area with an error toast: "Could not generate reset link" with body + Retry link + dismiss
5. **HTTP 404 page** — full page "Not found" with Go back / Dashboard buttons
