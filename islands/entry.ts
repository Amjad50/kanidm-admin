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
