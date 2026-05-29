// Entry point for client-side islands.
// Most of the app is server-rendered HTML; this file mounts Preact components
// only where genuinely interactive (Cmd+K palette, multi-step wizards, etc.).

import { h, render } from "preact";
import { CommandPalette } from "./command_palette";

// Mount the Cmd+K palette if the host element exists on this page.
const paletteHost = document.getElementById("cmd-palette-island");
if (paletteHost) {
  render(h(CommandPalette, {}), paletteHost);
}

// Clipboard handler — binds to [data-copy] elements anywhere in the document.
// Click → writes data-copy value to clipboard, briefly shows a checkmark.
document.addEventListener('click', async (event) => {
  const target = (event.target as HTMLElement | null)?.closest<HTMLElement>('[data-copy]');
  if (!target) return;
  const value = target.getAttribute('data-copy');
  if (!value) return;
  event.preventDefault();
  try {
    await navigator.clipboard.writeText(value);
    showCopiedFeedback(target);
  } catch (err) {
    console.warn('clipboard write failed', err);
  }
});

// Set-to-now handler — binds to [data-set-now] elements anywhere in the document.
// Click → sets nearest date + time inputs to current UTC date/time and selects the datetime radio.
document.addEventListener('click', (event) => {
  const target = (event.target as HTMLElement | null)?.closest<HTMLElement>('[data-set-now]');
  if (!target) return;
  event.preventDefault();
  const card = target.closest<HTMLElement>('[data-validity-card]');
  if (!card) return;
  const dateInput = card.querySelector<HTMLInputElement>('input[type=date]');
  const timeInput = card.querySelector<HTMLInputElement>('input[type=time]');
  const radioDatetime = card.querySelector<HTMLInputElement>('input[type=radio][value=datetime]');
  if (!dateInput || !timeInput) return;
  const now = new Date();
  dateInput.value = now.toISOString().slice(0, 10);
  timeInput.value = now.toISOString().slice(11, 16);
  if (radioDatetime) radioDatetime.checked = true;
});

// Email-rows handler — binds to any container marked [data-email-rows].
// Add button: [data-add-email][data-target="container-id"]
// Remove button inside a row: [data-remove]
// Star button inside a row: [data-make-primary]
// Clone template: <template id="${container-id}-tpl">
document.addEventListener('click', (event) => {
  const target = event.target as HTMLElement | null;
  if (!target) return;

  const addBtn = target.closest<HTMLElement>('[data-add-email]');
  if (addBtn) {
    const containerId = addBtn.getAttribute('data-target');
    if (!containerId) return;
    const rows = document.getElementById(containerId);
    if (!rows) return;
    const tpl = document.getElementById(`${containerId}-tpl`) as HTMLTemplateElement | null;
    let newRow: HTMLElement | null = null;
    if (tpl) {
      newRow = tpl.content.cloneNode(true) as HTMLElement;
      newRow = (newRow as DocumentFragment).firstElementChild as HTMLElement;
    } else {
      const first = rows.firstElementChild as HTMLElement | null;
      if (!first) return;
      newRow = first.cloneNode(true) as HTMLElement;
      const star = newRow.querySelector<HTMLElement>('[data-make-primary]');
      if (star) {
        star.classList.remove('text-warning');
        star.classList.add('text-tertiary', 'hover:text-warning');
        star.querySelector('svg')?.setAttribute('fill', 'none');
      }
      const input = newRow.querySelector<HTMLInputElement>('input[type=email]');
      if (input) input.value = '';
    }
    if (!newRow) return;
    rows.appendChild(newRow);
    newRow.querySelector<HTMLInputElement>('input[type=email]')?.focus();
    return;
  }

  const removeBtn = target.closest<HTMLElement>('[data-remove]');
  if (removeBtn) {
    const rows = removeBtn.closest<HTMLElement>('[data-email-rows]');
    if (!rows) return;
    removeBtn.closest<HTMLElement>('.flex')?.remove();
    return;
  }

  const starBtn = target.closest<HTMLElement>('[data-make-primary]');
  if (starBtn) {
    const rows = starBtn.closest<HTMLElement>('[data-email-rows]');
    if (!rows) return;
    const row = starBtn.closest<HTMLElement>('.flex');
    if (!row) return;

    rows.querySelectorAll<HTMLElement>('[data-make-primary]').forEach(b => {
      b.classList.remove('text-warning');
      b.classList.add('text-tertiary', 'hover:text-warning');
      b.querySelector('svg')?.setAttribute('fill', 'none');
    });

    starBtn.classList.add('text-warning');
    starBtn.classList.remove('text-tertiary', 'hover:text-warning');
    starBtn.querySelector('svg')?.setAttribute('fill', 'currentColor');

    if (rows.firstElementChild !== row) {
      rows.insertBefore(row, rows.firstElementChild);
    }
  }
});

function showCopiedFeedback(button: HTMLElement) {
  const svg = button.querySelector('svg');
  if (!svg) return;
  const w = svg.getAttribute('width') || '12';
  const h = svg.getAttribute('height') || '12';
  const original = svg.outerHTML;
  svg.outerHTML = `<svg width="${w}" height="${h}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-success"><path d="M20 6 9 17l-5-5"/></svg>`;
  setTimeout(() => {
    const newSvg = button.querySelector('svg');
    if (newSvg) newSvg.outerHTML = original;
  }, 1200);
}
