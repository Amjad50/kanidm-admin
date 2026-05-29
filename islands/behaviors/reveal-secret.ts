import { defineBehavior } from './index';

// Toggle between a masked placeholder and the real secret value on click.
// Markup contract:
//   <code data-secret-value="THE_SECRET">••••••••••••••••••••••••</code>
//   <button data-reveal-secret>…</button>
// The button must be the immediate next sibling of the <code> element.
// Clicking the button toggles between dots and the real value.

const MASK = '••••••••••••••••••••••••';

defineBehavior({
  selector: '[data-reveal-secret]',
  event: 'click',
  handler: (el, event) => {
    event.preventDefault();
    const code = el.previousElementSibling as HTMLElement | null;
    if (!code) return;
    const value = code.getAttribute('data-secret-value');
    if (!value) return;
    const showing = code.textContent === value;
    code.textContent = showing ? MASK : value;
  },
});
