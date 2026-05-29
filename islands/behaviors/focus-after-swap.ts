// Auto-focus elements marked with [data-focus-after-swap] on initial page
// load and after each HTMX swap settles.

function focusMarked(root: ParentNode): void {
  const el = root.querySelector<HTMLElement>("[data-focus-after-swap]");
  if (el) el.focus();
}

document.addEventListener("DOMContentLoaded", () => focusMarked(document));
document.body.addEventListener("htmx:afterSettle", (e) => {
  const detail = (e as CustomEvent).detail as
    | { target?: HTMLElement }
    | undefined;
  if (detail?.target) focusMarked(detail.target);
});
