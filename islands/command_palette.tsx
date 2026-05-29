// Cmd+K command palette. Placeholder implementation that demonstrates the
// island shape: small Preact component, controlled by global keybind, mounted
// once at app shell. Real fuzzy-search wires to a server endpoint later.

import { useEffect, useState } from "preact/hooks";

export function CommandPalette() {
  const [open, setOpen] = useState(false);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        setOpen((v) => !v);
      } else if (e.key === "Escape" && open) {
        setOpen(false);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open]);

  if (!open) return null;

  return (
    <div
      class="fixed inset-0 z-50 bg-black/60 backdrop-blur-sm flex items-start justify-center pt-24"
      onClick={() => setOpen(false)}
    >
      <div
        class="bg-zinc-900 border border-zinc-800 rounded-lg w-full max-w-xl shadow-2xl"
        onClick={(e) => e.stopPropagation()}
      >
        <input
          autofocus
          type="text"
          placeholder="Search people, groups, OAuth2 apps…"
          class="w-full bg-transparent px-4 py-3 text-zinc-100 placeholder-zinc-500 focus:outline-none border-b border-zinc-800"
        />
        <div class="p-2 text-zinc-500 text-sm">
          <p class="px-3 py-2">Type to search. (Wire to server endpoint in Phase 5.)</p>
        </div>
        <div class="px-4 py-2 text-xs text-zinc-500 border-t border-zinc-800 flex gap-3">
          <span>↑↓ navigate</span>
          <span>↵ select</span>
          <span>esc close</span>
        </div>
      </div>
    </div>
  );
}
