// Entry point for client-side islands.
// Most of the app is server-rendered HTML; this file mounts Preact components
// only where genuinely interactive (Cmd+K palette, multi-step wizards, etc.).

import { h, render } from "preact";
import { CommandPalette } from "./command_palette";
import { mountDropdowns } from "./dropdown";
import { mountPagination } from "./pagination";
import { mountToasts } from "./toast";

// Mount the Cmd+K palette if the host element exists on this page.
const paletteHost = document.getElementById("cmd-palette-island");
if (paletteHost) {
  render(h(CommandPalette, {}), paletteHost);
}

// Mount the global dropdown root — binds to every [data-dropdown] trigger.
mountDropdowns();

// Mount any [data-pagination] containers found on this page.
mountPagination();

// Mount the toast stack — listens for HX-Trigger "toast" events.
mountToasts();

// Surface the reauth modal instead of letting an expired session boot the
// user to a full 401 page mid-flow.
document.body.addEventListener('kanidm-reauth', () => {
  // @ts-expect-error htmx global is loaded by base.html
  window.htmx.ajax('GET', '/reauth', { target: '#overlay-slot', swap: 'innerHTML' });
});

// Theme toggle — persists to localStorage and reflects the current value on
// the [data-theme-set] tab buttons. The `.dark` class on <html> is applied
// by an inline script in base.html so the page paints in the right theme
// without a flash. Default is dark (matching shadcn convention where :root
// is light but this app's previous UX defaulted to dark).
const THEME_KEY = 'kanidm-admin-ui:theme';

function syncThemeTabs() {
  const current = document.documentElement.classList.contains('dark') ? 'dark' : 'light';
  document.querySelectorAll<HTMLElement>('[data-theme-set]').forEach((btn) => {
    const active = btn.dataset.themeSet === current;
    btn.classList.toggle('bg-card', active);
    btn.classList.toggle('text-foreground', active);
    btn.classList.toggle('text-muted-foreground', !active);
  });
}

syncThemeTabs();

document.addEventListener('click', (event) => {
  const target = (event.target as HTMLElement | null)?.closest<HTMLElement>('[data-theme-set]');
  if (!target) return;
  const theme = target.dataset.themeSet;
  if (theme !== 'light' && theme !== 'dark') return;
  event.preventDefault();
  document.documentElement.classList.toggle('dark', theme === 'dark');
  try {
    localStorage.setItem(THEME_KEY, theme);
  } catch {}
  syncThemeTabs();
});

// Open the command palette when a [data-open-palette] trigger is clicked.
document.addEventListener('click', (event) => {
  const target = (event.target as HTMLElement | null)?.closest<HTMLElement>('[data-open-palette]');
  if (!target) return;
  event.preventDefault();
  document.dispatchEvent(new CustomEvent('open-palette'));
});

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
        star.classList.add('text-muted-foreground', 'hover:text-warning');
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
      b.classList.add('text-muted-foreground', 'hover:text-warning');
      b.querySelector('svg')?.setAttribute('fill', 'none');
    });

    starBtn.classList.add('text-warning');
    starBtn.classList.remove('text-muted-foreground', 'hover:text-warning');
    starBtn.querySelector('svg')?.setAttribute('fill', 'currentColor');

    if (rows.firstElementChild !== row) {
      rows.insertBefore(row, rows.firstElementChild);
    }
  }
});

function showCopiedFeedback(button: HTMLElement) {
  const original = button.innerHTML;
  button.classList.add('text-success');
  button.innerHTML = `<span class="w-3.5 h-3.5 inline-flex shrink-0"><svg class="lucide" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg></span>`;
  setTimeout(() => {
    button.classList.remove('text-success');
    button.innerHTML = original;
  }, 1200);
}
