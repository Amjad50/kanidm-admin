import { defineBehavior } from './index';

// Eye toggle on a password input.
// Markup contract:
//   <div data-password-reveal>
//     <input type="password" ...>
//     <button type="button" data-password-reveal-toggle>
//       <svg data-eye-open>…</svg>
//       <svg data-eye-closed hidden>…</svg>
//     </button>
//   </div>

defineBehavior({
  selector: '[data-password-reveal-toggle]',
  event: 'click',
  handler: (el, event) => {
    event.preventDefault();
    const wrap = el.closest<HTMLElement>('[data-password-reveal]');
    if (!wrap) return;
    const input = wrap.querySelector<HTMLInputElement>('input[type="password"], input[type="text"]');
    if (!input) return;
    const showing = input.type === 'text';
    input.type = showing ? 'password' : 'text';
    el.setAttribute('aria-label', showing ? 'Show password' : 'Hide password');
    const open = el.querySelector<HTMLElement>('[data-eye-open]');
    const closed = el.querySelector<HTMLElement>('[data-eye-closed]');
    if (open && closed) {
      open.hidden = !showing;
      closed.hidden = showing;
    }
  },
});
