// Pin viewport scroll position across HTMX swaps.
//
// When HTMX replaces an element (typically a card), the browser can shift
// the viewport — either because the swapped fragment changes height above
// the user's scroll position, or because a freshly-inserted input briefly
// becomes the active element. The result is the page jumps to the top on
// what should be an in-place update.
//
// We capture window.scrollY just before the swap fires and restore it on
// the next two animation frames (one for the swap, one for any settle
// reflow). Opt-out via hx-swap="… scroll:smooth" on the trigger element.

let pendingScrollY: number | null = null;

document.body.addEventListener("htmx:beforeSwap", () => {
  pendingScrollY = window.scrollY;
});

document.body.addEventListener("htmx:afterSwap", () => {
  if (pendingScrollY === null) return;
  const y = pendingScrollY;
  pendingScrollY = null;
  requestAnimationFrame(() => {
    window.scrollTo({ top: y, behavior: "instant" as ScrollBehavior });
    requestAnimationFrame(() => {
      window.scrollTo({ top: y, behavior: "instant" as ScrollBehavior });
    });
  });
});
