// Entry point for client-side islands and behaviors.
//
// Most of the app is server-rendered HTML. This file mounts:
//   - Preact islands for genuinely stateful UI (cmd+K palette, dropdowns
//     driven by JSON, toast stack, pagination)
//   - Behaviors (delegated DOM enhancement) — see ./behaviors/README.md

import { h, render } from "preact";
import { CommandPalette } from "./command_palette";
import { mountDropdowns } from "./dropdown";
import { mountPagination } from "./pagination";
import { mountToasts } from "./toast";
import { mountBehaviors } from "./behaviors";

// Behaviors register themselves via side-effect imports.
import "./behaviors/copy";
import "./behaviors/theme";
import "./behaviors/palette-open";
import "./behaviors/set-now";
import "./behaviors/email-rows";
import "./behaviors/row-href";
import "./behaviors/bind-disabled";
import "./behaviors/reveal-secret";
import "./behaviors/warn-duplicate";

mountBehaviors();
mountDropdowns();
mountPagination();
mountToasts();

const paletteHost = document.getElementById("cmd-palette-island");
if (paletteHost) {
  render(h(CommandPalette, {}), paletteHost);
}

// Reauth modal — surfaced from session-expiry events instead of a 401 redirect.
document.body.addEventListener('kanidm-reauth', () => {
  // @ts-expect-error htmx global is loaded by base.html
  window.htmx.ajax('GET', '/reauth', { target: '#overlay-slot', swap: 'innerHTML' });
});
