import { defineBehavior } from './index';

defineBehavior({
  selector: '[data-add-email]',
  event: 'click',
  handler: (el) => {
    const containerId = el.getAttribute('data-target');
    if (!containerId) return;
    const rows = document.getElementById(containerId);
    if (!rows) return;
    const tpl = document.getElementById(`${containerId}-tpl`) as HTMLTemplateElement | null;
    let newRow: HTMLElement | null = null;
    if (tpl) {
      const frag = tpl.content.cloneNode(true) as DocumentFragment;
      newRow = frag.firstElementChild as HTMLElement;
    } else {
      const first = rows.firstElementChild as HTMLElement | null;
      if (!first) return;
      newRow = first.cloneNode(true) as HTMLElement;
      const star = newRow.querySelector<HTMLElement>('[data-make-primary]');
      if (star) {
        star.classList.remove('text-warning');
        star.classList.add('text-muted-foreground', 'hover:text-warning');
        star.querySelector('svg')?.setAttribute('fill', 'none');
      }
      const input = newRow.querySelector<HTMLInputElement>('input[type=email]');
      if (input) input.value = '';
    }
    if (!newRow) return;
    rows.appendChild(newRow);
    newRow.querySelector<HTMLInputElement>('input[type=email]')?.focus();
  },
});

defineBehavior({
  selector: '[data-remove]',
  event: 'click',
  handler: (el) => {
    const rows = el.closest<HTMLElement>('[data-email-rows]');
    if (!rows) return;
    el.closest<HTMLElement>('.flex')?.remove();
  },
});

defineBehavior({
  selector: '[data-make-primary]',
  event: 'click',
  handler: (el) => {
    const rows = el.closest<HTMLElement>('[data-email-rows]');
    if (!rows) return;
    const row = el.closest<HTMLElement>('.flex');
    if (!row) return;

    rows.querySelectorAll<HTMLElement>('[data-make-primary]').forEach(b => {
      b.classList.remove('text-warning');
      b.classList.add('text-muted-foreground', 'hover:text-warning');
      b.querySelector('svg')?.setAttribute('fill', 'none');
    });

    el.classList.add('text-warning');
    el.classList.remove('text-muted-foreground', 'hover:text-warning');
    el.querySelector('svg')?.setAttribute('fill', 'currentColor');

    if (rows.firstElementChild !== row) {
      rows.insertBefore(row, rows.firstElementChild);
    }
  },
});
