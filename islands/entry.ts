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
