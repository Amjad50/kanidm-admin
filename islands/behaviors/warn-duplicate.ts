import { defineBehavior } from './index';

// Toggle visibility of a warning element when an input's trimmed,
// lowercased value matches any item in a JSON-encoded list of existing
// values. Used by the scope-map modal to flag "this group already has a
// scope map — saving will overwrite".
//
// Markup contract:
//   <input data-warn-duplicate
//          data-warn-existing='["alice@example.com","bob@example.com"]'
//          data-warn-target="#some-warning-element" />
//   <div id="some-warning-element" class="hidden">…</div>
//
// The warning starts hidden (caller's responsibility — add `hidden` class).
// On input AND on initial load, the behavior toggles the `hidden` class
// on the target based on whether the value matches.

function parseList(raw: string | null): string[] {
  if (!raw) return [];
  try {
    const parsed = JSON.parse(raw);
    return Array.isArray(parsed) ? parsed.map(String) : [];
  } catch {
    return [];
  }
}

function check(input: HTMLInputElement): void {
  const targetSel = input.getAttribute('data-warn-target');
  if (!targetSel) return;
  const target = document.querySelector<HTMLElement>(targetSel);
  if (!target) return;
  const existing = parseList(input.getAttribute('data-warn-existing'));
  const val = input.value.trim().toLowerCase();
  const found = existing.some((g) => g.toLowerCase() === val);
  target.classList.toggle('hidden', !found);
}

defineBehavior({
  selector: '[data-warn-duplicate]',
  event: 'input',
  handler: (el) => {
    check(el as HTMLInputElement);
  },
});

// Initial pass on every matching input at module load — the scope-map modal
// can open with a pre-filled value (edit-mode rare, but defensive).
document.addEventListener('DOMContentLoaded', () => {
  document.querySelectorAll<HTMLInputElement>('[data-warn-duplicate]').forEach(check);
});

// Also re-run on HTMX swaps that bring [data-warn-duplicate] into the DOM.
document.body.addEventListener('htmx:afterSwap', (e) => {
  const target = (e as CustomEvent).detail?.target as HTMLElement | undefined;
  if (!target) return;
  target.querySelectorAll<HTMLInputElement>('[data-warn-duplicate]').forEach(check);
});
