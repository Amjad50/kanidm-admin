import { defineBehavior } from './index';

// One-way binding: input's value must exactly match a token, otherwise the
// referenced target button is disabled. Used by the destructive-confirm modal
// to gate the confirm button behind "type the SPN to confirm".
//
// Markup:
//   <input data-bind-source
//          data-bind-expected="alice@example.com"
//          data-bind-target="#confirm-submit-foo" />
//
// Exact match by design — pasting trailing whitespace must not silently
// satisfy a destructive action.

defineBehavior({
  selector: '[data-bind-source]',
  event: 'input',
  handler: (el) => {
    const input = el as HTMLInputElement;
    const expected = el.getAttribute('data-bind-expected') ?? '';
    const targetSel = el.getAttribute('data-bind-target');
    if (!targetSel) return;
    const target = document.querySelector<HTMLButtonElement>(targetSel);
    if (!target) return;
    target.disabled = input.value !== expected;
  },
});
