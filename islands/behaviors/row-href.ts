import { defineBehavior } from './index';

// Whole-row click navigates to the row's data-row-href URL.
// Modifier keys (ctrl/cmd/shift) and middle-click bail out so the browser's
// open-in-new-tab behavior on inner anchors keeps working. Clicks on inner
// interactive elements (buttons, anchors, inputs, labels) also bail.
//
// Inner elements that want to opt-out of row-nav even though they aren't
// natively interactive can carry [data-no-row-nav].

function shouldSkip(e: MouseEvent): boolean {
  return e.ctrlKey || e.metaKey || e.shiftKey || e.button === 1;
}

function clickedInteractive(target: EventTarget | null): boolean {
  // Use Element (not HTMLElement) because SVG nodes inside buttons/anchors
  // are Elements but not HTMLElements. A strict HTMLElement check makes
  // closest() skip the climb when the user clicks on an icon's <svg>/<path>,
  // causing row-nav to fire on top of the kebab/icon-button click.
  if (!(target instanceof Element)) return false;
  return !!target.closest('button, a, input, label, [data-no-row-nav]');
}

defineBehavior({
  selector: '[data-row-href]',
  event: 'click',
  handler: (el, event) => {
    const me = event as MouseEvent;
    if (shouldSkip(me)) return;
    if (clickedInteractive(me.target)) return;
    const href = el.getAttribute('data-row-href');
    if (!href) return;
    window.location.assign(href);
  },
});

// Middle-click → new tab.
defineBehavior({
  selector: '[data-row-href]',
  event: 'auxclick',
  handler: (el, event) => {
    const me = event as MouseEvent;
    if (me.button !== 1) return;
    if (clickedInteractive(me.target)) return;
    const href = el.getAttribute('data-row-href');
    if (!href) return;
    me.preventDefault();
    window.open(href, '_blank', 'noopener');
  },
});
