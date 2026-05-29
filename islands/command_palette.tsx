// Cmd+K command palette. Fuzzy-searches across people, groups, oauth2 apps via
// the existing list endpoints' JSON branch, and exposes a fixed set of action
// commands (Create person, Go to profile, ...). Keyboard nav, recents in
// localStorage. Opens via Cmd/Ctrl+K or a fired `open-palette` DOM event.

import { useEffect, useMemo, useRef, useState } from "preact/hooks";

type Kind = "action" | "person" | "group" | "oauth2";

interface PaletteItem {
  kind: Kind;
  label: string;
  subtitle: string;
  href: string;
}

interface ApiResponse {
  items: PaletteItem[];
}

const RECENTS_KEY = "kanidm-admin-ui:recents";
const RECENTS_MAX = 8;
const DEBOUNCE_MS = 180;

const SECTION_TITLE: Record<Kind, string> = {
  action: "Actions",
  person: "People",
  group: "Groups",
  oauth2: "OAuth2 apps",
};

const ENTITY_KINDS: Exclude<Kind, "action">[] = ["person", "group", "oauth2"];
const ENTITY_PATH: Record<Exclude<Kind, "action">, string> = {
  person: "/people",
  group: "/groups",
  oauth2: "/oauth2",
};

// Hand-ordered: verbs people most often want first.
const ACTIONS: PaletteItem[] = [
  { kind: "action", label: "Create person", subtitle: "Add a new account", href: "/people/new" },
  { kind: "action", label: "Create group", subtitle: "Define a new group", href: "/groups/new" },
  {
    kind: "action",
    label: "Create OAuth2 application",
    subtitle: "Configure SSO for a service",
    href: "/oauth2/new",
  },
  { kind: "action", label: "My profile", subtitle: "/me", href: "/me" },
  { kind: "action", label: "My sessions", subtitle: "/me/sessions", href: "/me/sessions" },
];

function matchesQuery(it: PaletteItem, q: string): boolean {
  const needle = q.toLowerCase();
  return (
    it.label.toLowerCase().includes(needle) ||
    it.subtitle.toLowerCase().includes(needle)
  );
}

function readRecents(): PaletteItem[] {
  try {
    const raw = localStorage.getItem(RECENTS_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw);
    if (!Array.isArray(parsed)) return [];
    return parsed.filter(
      (it): it is PaletteItem =>
        it &&
        typeof it.kind === "string" &&
        typeof it.label === "string" &&
        typeof it.subtitle === "string" &&
        typeof it.href === "string",
    );
  } catch {
    return [];
  }
}

function writeRecents(items: PaletteItem[]) {
  try {
    localStorage.setItem(RECENTS_KEY, JSON.stringify(items.slice(0, RECENTS_MAX)));
  } catch {
    // ignore quota / disabled storage
  }
}

function pushRecent(item: PaletteItem) {
  const current = readRecents();
  const deduped = current.filter((r) => r.href !== item.href);
  deduped.unshift(item);
  writeRecents(deduped);
}

export function CommandPalette() {
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const [entityItems, setEntityItems] = useState<PaletteItem[]>([]);
  const [loading, setLoading] = useState(false);
  const [highlight, setHighlight] = useState(0);
  const [recents, setRecents] = useState<PaletteItem[]>([]);
  const inputRef = useRef<HTMLInputElement>(null);
  const abortRef = useRef<AbortController | null>(null);

  // Global hotkeys + external open-palette event.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        setOpen((v) => !v);
      }
    };
    const onOpen = () => setOpen(true);
    window.addEventListener("keydown", onKey);
    document.addEventListener("open-palette", onOpen);
    return () => {
      window.removeEventListener("keydown", onKey);
      document.removeEventListener("open-palette", onOpen);
    };
  }, []);

  // Reset state when opened; focus input.
  useEffect(() => {
    if (!open) {
      abortRef.current?.abort();
      abortRef.current = null;
      return;
    }
    setQuery("");
    setEntityItems([]);
    setHighlight(0);
    setRecents(readRecents());
    queueMicrotask(() => inputRef.current?.focus());
  }, [open]);

  // Debounced fetch of entity matches when there's a query.
  useEffect(() => {
    if (!open) return;
    const trimmed = query.trim();
    if (!trimmed) {
      abortRef.current?.abort();
      abortRef.current = null;
      setEntityItems([]);
      setLoading(false);
      setHighlight(0);
      return;
    }

    const timer = window.setTimeout(() => {
      abortRef.current?.abort();
      const controller = new AbortController();
      abortRef.current = controller;
      setLoading(true);

      const qp = encodeURIComponent(trimmed);
      const fetchKind = (path: string) =>
        fetch(`${path}?q=${qp}`, {
          signal: controller.signal,
          headers: { Accept: "application/json" },
          credentials: "same-origin",
        })
          .then((r) =>
            r.ok ? (r.json() as Promise<ApiResponse>) : { items: [] as PaletteItem[] },
          )
          .catch(() => ({ items: [] as PaletteItem[] }));

      Promise.all(ENTITY_KINDS.map((k) => fetchKind(ENTITY_PATH[k])))
        .then((results) => {
          if (controller.signal.aborted) return;
          setEntityItems(results.flatMap((r) => r.items));
          setHighlight(0);
          setLoading(false);
        })
        .catch(() => {
          if (controller.signal.aborted) return;
          setLoading(false);
        });
    }, DEBOUNCE_MS);

    return () => window.clearTimeout(timer);
  }, [query, open]);

  // Build the displayed sections.
  //   empty query  → Recent, Actions
  //   typed query  → matching Actions first (verbs), then People, Groups, OAuth2 apps
  const sections = useMemo<{ title: string; items: PaletteItem[] }[]>(() => {
    const out: { title: string; items: PaletteItem[] }[] = [];
    const trimmed = query.trim();
    if (!trimmed) {
      if (recents.length > 0) out.push({ title: "Recent", items: recents });
      out.push({ title: SECTION_TITLE.action, items: ACTIONS });
      return out;
    }
    const matchedActions = ACTIONS.filter((a) => matchesQuery(a, trimmed));
    if (matchedActions.length > 0) {
      out.push({ title: SECTION_TITLE.action, items: matchedActions });
    }
    for (const kind of ENTITY_KINDS) {
      const group = entityItems.filter((it) => it.kind === kind);
      if (group.length > 0) out.push({ title: SECTION_TITLE[kind], items: group });
    }
    return out;
  }, [entityItems, recents, query]);

  // Flat list for keyboard nav, in display order.
  const visible = useMemo<PaletteItem[]>(
    () => sections.flatMap((s) => s.items),
    [sections],
  );

  useEffect(() => {
    if (highlight >= visible.length) setHighlight(0);
  }, [visible.length, highlight]);

  const choose = (it: PaletteItem) => {
    pushRecent(it);
    setOpen(false);
    window.location.href = it.href;
  };

  const onKeyDown = (e: KeyboardEvent) => {
    if (e.key === "Escape") {
      e.preventDefault();
      setOpen(false);
      return;
    }
    if (e.key === "ArrowDown") {
      e.preventDefault();
      if (visible.length === 0) return;
      setHighlight((h) => (h + 1) % visible.length);
      return;
    }
    if (e.key === "ArrowUp") {
      e.preventDefault();
      if (visible.length === 0) return;
      setHighlight((h) => (h - 1 + visible.length) % visible.length);
      return;
    }
    if (e.key === "Enter") {
      e.preventDefault();
      const pick = visible[highlight];
      if (pick) choose(pick);
    }
  };

  if (!open) return null;

  const trimmed = query.trim();
  const showingRecents = trimmed.length === 0;
  let runningIndex = 0;

  return (
    <div
      class="fixed inset-0 z-50 flex items-start justify-center pt-24 bg-background/70 backdrop-blur-sm"
      onClick={() => setOpen(false)}
      role="presentation"
    >
      <div
        class="w-full max-w-xl rounded-lg border border-border bg-card shadow-2xl"
        onClick={(e) => e.stopPropagation()}
        role="dialog"
        aria-label="Command palette"
      >
        <input
          ref={inputRef}
          type="text"
          value={query}
          onInput={(e) => setQuery((e.currentTarget as HTMLInputElement).value)}
          onKeyDown={onKeyDown}
          placeholder="Search or run a command…"
          class="w-full border-b border-border bg-transparent px-4 py-3 text-foreground placeholder:text-muted-foreground focus:outline-none"
          aria-label="Search"
          autocomplete="off"
          spellcheck={false}
        />

        <div class="max-h-[60vh] overflow-y-auto p-2">
          {sections.length === 0 && (
            <p class="px-3 py-6 text-center text-sm text-muted-foreground">
              {loading ? "Searching…" : "No matches."}
            </p>
          )}

          {sections.map((section) => (
            <div key={section.title} class="mb-2 last:mb-0">
              <div class="px-3 pb-1 pt-2 text-xs font-medium uppercase tracking-wide text-muted-foreground">
                {section.title}
              </div>
              <ul role="listbox">
                {section.items.map((it) => {
                  const idx = runningIndex++;
                  const isActive = idx === highlight;
                  return (
                    <li
                      key={`${it.kind}:${it.href}`}
                      role="option"
                      aria-selected={isActive}
                      class={
                        "flex cursor-pointer items-center justify-between gap-3 rounded px-3 py-2 " +
                        (isActive ? "bg-primary-soft text-primary" : "text-foreground hover:bg-accent")
                      }
                      onMouseEnter={() => setHighlight(idx)}
                      onClick={() => choose(it)}
                    >
                      <div class="min-w-0 flex-1">
                        <div class="truncate text-sm">{it.label}</div>
                        {it.subtitle && (
                          <div
                            class={
                              "truncate text-xs " +
                              (isActive ? "text-primary" : "text-muted-foreground")
                            }
                          >
                            {it.subtitle}
                          </div>
                        )}
                      </div>
                      <span
                        class={
                          "shrink-0 rounded-pill px-2 py-0.5 text-[10px] uppercase tracking-wide " +
                          (isActive
                            ? "bg-primary text-primary-foreground"
                            : "bg-popover text-muted-foreground")
                        }
                      >
                        {SECTION_TITLE[it.kind]}
                      </span>
                    </li>
                  );
                })}
              </ul>
            </div>
          ))}
        </div>

        <div class="flex gap-3 border-t border-border px-4 py-2 text-xs text-muted-foreground">
          <span>↑↓ navigate</span>
          <span>↵ select</span>
          <span>esc close</span>
          {loading && !showingRecents && <span class="ml-auto">loading…</span>}
        </div>
      </div>
    </div>
  );
}
