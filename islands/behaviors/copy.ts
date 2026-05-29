import { defineBehavior } from './index';

defineBehavior({
  selector: '[data-copy]',
  event: 'click',
  handler: async (el, event) => {
    const value = el.getAttribute('data-copy');
    if (!value) return;
    event.preventDefault();
    try {
      await navigator.clipboard.writeText(value);
      flash(el);
    } catch (err) {
      console.warn('clipboard write failed', err);
    }
  },
});

function flash(button: HTMLElement) {
  const original = button.innerHTML;
  button.classList.add('text-success');
  button.innerHTML = `<span class="w-3.5 h-3.5 inline-flex shrink-0"><svg class="lucide" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg></span>`;
  setTimeout(() => {
    button.classList.remove('text-success');
    button.innerHTML = original;
  }, 1200);
}
