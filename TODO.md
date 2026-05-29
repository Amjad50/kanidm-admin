# TODO

Running list of follow-up work that isn't blocking any current phase but
should be considered.

## Preact migration candidates

We currently use Preact for the cmd+K palette and (newly, post-Phase 2.5)
for row-level kebab dropdowns. Other client-side interactions are HTMX or
inline DOM script. The bar for moving something to Preact is: **client state
that's awkward to express server-side OR positioning/keyboard logic that
needs JS anyway**.

Things to revisit:

- **Modals.** Today modals live in `#overlay-slot`, populated by HTMX. Backdrop
  click clears via `hx-trigger="click[target===this]"`. This works but has two
  papercuts:
  1. Esc-to-close needs a separate global listener we haven't added.
  2. Stacking (modal-on-modal) isn't possible — only one `#overlay-slot`.
  
  A `<ModalProvider>` Preact root that owns the modal stack would fix both
  and let callers do `modal.open(<...body...>)` from any island. The trade is
  more JS, plus all the current modal-flow handlers need to return modal
  bodies suitable for being rendered inside a Preact-owned shell rather than
  a self-contained modal HTML fragment.
  
  Decision: defer. The papercuts are small and the existing flow is well
  understood.

- **Toasts.** Phase 5 plans a Preact toast island. Server triggers via
  `HX-Trigger: {"toast": {...}}`. Hasn't been implemented yet.

- **Email row interactivity** (`[data-email-rows]`). Currently a vanilla
  delegated DOM handler in `islands/entry.ts`. Works fine. Would be Preact-y
  to make it a `<EmailList>` component with React state, but the current
  implementation is ~40 lines and does the job. Don't refactor unless we
  hit an actual bug.

- **Multi-select for scope/claim maps (Phase 4).** When 4E and 4F land they'll
  need a "pick a group + pick scopes" interaction. That's a strong Preact
  candidate — combobox + chip remove + free-text custom value. Plan to build
  fresh in Preact during Phase 4.

## Other cleanup

- **Dead struct fields in groups module.** `MemberRow.displayname`,
  `MemberRow.spn_or_id`, `GroupListRow.member_count`, `GroupListRow.has_policy`
  are all warned by the compiler. Either render them or delete them. Carried
  over from Phase 3.

- **`AppError::Other(anyhow::Error)` is never constructed.** Either start
  using it or remove the variant.

- **People Overview "Credentials summary"** card duplicates info from the
  Credentials tab. Consider trimming to just primary + passkey count, with
  "see all on Credentials tab" link.

- **Avatar deterministic colors.** Right now everyone gets `bg-accent-soft`.
  Hash the SPN into one of ~8 muted backgrounds for visual scannability.

- **Bulk-select checkboxes** on the people list — currently rendered but not
  wired. Either implement select-many actions or remove. (Phase 2.5 removes
  them.)
