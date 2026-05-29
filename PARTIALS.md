# Template partials — registry & policy

Living index of every shared template fragment, every fragment we plan to extract, and every duplicated chunk we've identified but not yet extracted. Subagents working on a task MUST read this file before writing templates, and MUST update it at the end of their task.

## Rule for subagents

1. **Before writing a template chunk**, scan this file for an existing or proposed partial that matches.
2. **If the chunk is `extracted`**, use `{% include "<path>" %}` or the documented include strategy.
3. **If the chunk is `proposed`**, you are the owning task — extract it now as part of your work. Update the row's `Status` to `extracted` and `Path` to the real file.
4. **If the chunk is `candidate`**, decide: extract it (and flip to `extracted`), or duplicate it inline. If you duplicate, you must add a row noting the second call-site.
5. **If you spot a new pattern** that will repeat (or already repeats), add a row at the bottom under `## Newly discovered (this task)`. Move it to the main table when committing.
6. **Status definitions:**
   - **proposed** — first task that needs the pattern owns the extraction. Not yet on disk.
   - **candidate** — exists in 2+ places now; extract when convenient (e.g., when a task touches both call-sites).
   - **extracted** — done. Path is canonical.
   - **rejected** — considered and decided against (e.g., scope too narrow to share). Don't re-propose without new evidence.
7. **Report section.** End every subagent report with a "Partials touched / proposed" block listing rows you added or flipped.

## Include strategy

Two patterns are in use:

- **Askama `{% include %}` with parent scope.** Cheap, but the included template reads variables from the calling template's struct. Risk of name collisions when the same partial is used multiple times in one template. Use for icons and small fragments where there's exactly one instance per parent.
- **Nested Askama `Template` struct → render to string → `{{ rendered|safe }}`.** Heavier, but each instance gets isolated context. Use for any partial that needs to be rendered multiple times per parent (e.g., form fields, list rows).

Each row below documents which strategy applies.

---

## Registry

| # | Partial | Status | Strategy | Path | Used by | Owner |
|---|---|---|---|---|---|---|
| 1 | Copy-to-clipboard SVG icon | extracted | `{% include %}` | `templates/icons/_copy.html` | 2B | 2B |
| 2 | Form field (label + input + optional suffix + helper + error) | extracted | nested `Template` + `\|safe` | `templates/people/_form_field.html` (struct: `FormField` in `src/handlers/people/create.rs`; fields: `multiline: bool`, `rows: u32` branch `<textarea>` vs `<input>` — existing callers use `multiline: false, rows: 0`) | 2C, 2D, 2G (multiline variant), 3*, 4* | 2C |
| 3 | Tab nav (people detail tabs, OOB-swappable) | extracted | `{% include %}`, with `oob: bool` field | `templates/people/_tabs_nav.html` | 2B | 2B |
| 4 | Modal frame (overlay backdrop + dialog card + close affordance) | extracted | nested `Template` + `\|safe` (body is markup) | `templates/partials/_modal.html` (struct: `Modal` in `src/views/partials.rs`) | 2E, 2F, 3E, 4J, more | 2E |
| 5 | One-time-show secret reveal (value + copy + QR + expiry + optional action row) | extracted | nested `Template` + `\|safe` | `templates/partials/_one_time_secret.html` (struct: `OneTimeSecret` in `src/views/partials.rs`) | 2F, 2H, 4D | 2F |
| 6 | Toast notification (success / warning / danger) | proposed | nested `Template` + `\|safe`, triggered via `HX-Trigger` | `templates/partials/_toast.html` (planned) | 5E and any mutation handler | 5E |
| 7 | Status badge (pill with dot + label + soft-bg + colored text) | candidate | `{% include %}` (caller has `status_label` + `status_badge_classes` + `status_dot_classes` in scope) | `templates/partials/_status_badge.html` (proposed path) | 2A (`_rows.html`), 2B (detail header) | TBD (whoever touches both next) |
| 8 | Empty-state row inside a table (`<tr>` spanning all cols with friendly message) | candidate | `{% include %}` with parent providing message + colspan | `templates/partials/_table_empty_row.html` (proposed) | 2A, 3A, 4A | TBD |
| 9 | Kebab "more actions" button (icon-only `<button>` shell) | candidate | `{% include %}`, with `aria_label` in scope | `templates/icons/_kebab.html` (icon only — surrounding `<button>` stays local) | 2A (`_rows.html`), 2B (header) | TBD |
| 10 | Pagination block (prev / 1…N / next + per-page select) | candidate | nested `Template` + `\|safe`, fed paging data | `templates/partials/_pagination.html` (proposed) | 2A; reused by 3A, 4A | TBD (extract when 3A copies the markup) |
| 11 | Card frame (`<section>` with surface bg + border + shadow + padding) | rejected | n/a | n/a | everywhere | n/a — Tailwind utility soup is short enough (5 classes); extracting into a partial complicates more than it simplifies |
| 12 | "Copy button" composite (icon-only `<button data-copy="...">` wrapping the copy SVG) | candidate | nested `Template` + `\|safe`, takes a value | `templates/partials/_copy_button.html` (proposed) | 2B (UUID, SPN), 2E (SPN in destructive confirm), expected in 2F, 2H, 4D | TBD (extract when a third call-site lands) |
| 13 | List-row drag handle SVG (six-dot grip) | rejected | n/a | n/a | Drag reorder deferred to future task; edit form uses star-to-top instead | 2D |
| 14 | Star icon (filled vs outline, for primary-email indicator) | rejected | n/a | n/a | Inlined in `templates/people/edit.html` email rows loop — only one call-site so far; extract if 4D reuses it | 2D |
| 15 | Remove "×" button (small icon-only delete affordance, used in list rows / chips) | extracted | `{% include %}` icon-only | `templates/icons/_x.html` | 2B (group chip), 2D (email row), 2E (modal close button), 3C (member chip), 4E (scope row), 4F (claim row) | 2E |
| 16 | Confirm-destructive modal body (type-NAME-to-confirm input + confirm button disabled until match) | extracted | nested `Template` + `\|safe` | `templates/partials/_destructive_confirm.html` (struct: `DestructiveConfirm` in `src/views/partials.rs`) | 2E, 3E, 4J | 2E |
| 17 | Email-row interactive script (add / remove / star-to-primary JS) | extracted | global delegated handler in `islands/entry.ts` — attaches to `[data-email-rows]` containers; add button uses `[data-add-email][data-target="<container-id>"]`; template element named `<container-id>-tpl`; remove rows via `[data-remove]`; star/primary via `[data-make-primary]` | `islands/entry.ts` (see JS companions row 6) | people/create, people/edit, groups/create, groups/edit | this task |
| 18 | Identity row (avatar initials + display name + SPN mono) | extracted | nested `Template` + `\|safe` | `templates/partials/_identity_row.html` (struct: `IdentityRow` in `src/views/partials.rs`) | 2E (delete modal), 3E (delete group), 4J (delete oauth2) | 2E |
| 19 | Destructive-action footer (Cancel + disabled confirm button wired to `_destructive_confirm` input) | extracted | nested `Template` + `\|safe` | `templates/partials/_delete_footer.html` (struct: `DeleteFooter` with `action_url: String`, `confirm_label: String`, `input_id: String` in `src/views/partials.rs`) | 2E, 3A (group delete), 3C (member purge); reusable by 4J | 2E |
| 20 | SPN initials helper (`spn → "AB"` for avatar badges) | extracted (groups only) | Rust fn, not a template | `pub(super) fn spn_initials(spn: &str) -> String` in `src/handlers/groups/common.rs` | 3B (overview member chips), 3C (member rows) | 3 |
| 21 | Accent ring shadow (search-match-highlight box-shadow using accent color, 20% mix) | extracted | Tailwind `shadow-accent-ring` utility backed by `--shadow-accent-ring` token in `styles/tokens.css` | 3A (groups list row search hit) | 3 |
| 22 | Policy error fragment (inline danger banner returned from HTMX policy set/reset failures, Askama-escaped) | extracted | inline `#[derive(Template)] #[template(source = …)]` in `src/handlers/groups/policy.rs` (`PolicyErrorFragment`) | 3D | 3 |
| 23 | Members error slot (HTMX `hx-swap-oob` target above members table for add/remove errors) | extracted | `<div id="members-error">` slot in `templates/groups/_tab_members.html`; OOB fragment returned by `add`/`remove` handlers in `src/handlers/groups/members.rs` | 3C; pattern reusable for any tab subscreen with a list mutation | 3 |

| 24 | `EmailRow` struct + `emails_to_rows` helper | extracted | Rust shared type | `src/handlers/common.rs` | people/create, people/edit, groups/create, groups/edit | this task |

---

## JS / island companions

Some partials need a tiny JS counterpart. List them here so we don't lose track.

| # | Companion | Status | Where it lives | Triggers |
|---|---|---|---|---|
| 1 | Clipboard copy handler (binds to `[data-copy]` selector, calls `navigator.clipboard.writeText`, shows brief "copied" hint) | extracted | `islands/entry.ts` global behavior, no island root needed | Click on any `data-copy` element across the app |
| 2 | Modal-close shortcut (Esc key clears `#overlay-slot`) | proposed | `islands/entry.ts` | Esc key while overlay-slot non-empty |
| 3 | Toast renderer | proposed | `islands/toast.tsx` (Preact island bound to `#toast-stack`) | `htmx:trigger` event `toast` payload |
| 4 | Datetime picker with keyword shortcuts (now / never / clear) | proposed | `islands/datetime_keyword.tsx` (mounted by data-attr — v1 skipped: 2J uses native date/time inputs + global [data-set-now] handler in row 5 instead; Preact island deferred) | 2J validity form |
| 5 | `[data-set-now]` click handler (sets nearest date+time inputs to UTC now, selects datetime radio) | extracted | `islands/entry.ts` global delegated listener | Click on any `[data-set-now]` element — used in validity cards |
| 6 | Email-rows add/remove/star handler (delegated, `[data-email-rows]` container) | extracted | `islands/entry.ts` global delegated listener on document | `[data-add-email]`, `[data-remove]`, `[data-make-primary]` within any `[data-email-rows]` container — used in people/create, people/edit, groups/create, groups/edit |

---

## Newly discovered (this task)

*(Subagents append rows here during a task. Controller moves them into the main table at task-merge time.)*

---

## Conflict-avoidance notes

- Two tasks running in parallel worktrees should never both flip the same row from `proposed`/`candidate` → `extracted`. Coordinate by claiming a row's ownership in this file before starting work.
- If two tasks need the same partial and neither was the original owner, the **earlier-numbered task** extracts it; the later task uses it.
- Adding new entries to the registry table is conflict-prone (last-write-wins). When a subagent adds a row, it should pick the next free `#` and document the addition clearly in its commit so a merge can resolve duplicates by hand.
