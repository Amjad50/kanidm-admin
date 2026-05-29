// Generic kebab / overflow dropdown menu.
//
// Server-side: render any trigger button as
//   <button data-dropdown='{"items": [...]}' ...>kebab svg</button>
//
// The JSON config:
//   { items: DropdownItem[], align?: "left" | "right" (default "right") }
//
//   type DropdownItem =
//     | { kind: "link", label: string, href: string, danger?: boolean, icon?: "..." }
//     | { kind: "htmx", label: string, hxGet?: string, hxPost?: string,
//                      hxTarget?: string, hxSwap?: string, hxConfirm?: string,
//                      danger?: boolean, icon?: "..." }
//     | { kind: "divider" }
//
// Mount once at app start; this module finds every [data-dropdown] button and
// binds a click handler that toggles a positioned <Menu> element rendered into
// a single global root.

import { h, render } from "preact";
import { useEffect, useLayoutEffect, useRef, useState } from "preact/hooks";

type Align = "left" | "right";

type LinkItem = {
  kind: "link";
  label: string;
  href: string;
  danger?: boolean;
  icon?: IconKey;
};

type HtmxItem = {
  kind: "htmx";
  label: string;
  hxGet?: string;
  hxPost?: string;
  hxTarget?: string;
  hxSwap?: string;
  hxConfirm?: string;
  danger?: boolean;
  icon?: IconKey;
};

type DividerItem = { kind: "divider" };

type DropdownItem = LinkItem | HtmxItem | DividerItem;

type DropdownConfig = {
  items: DropdownItem[];
  align?: Align;
};

type IconKey =
  | "edit"
  | "delete"
  | "reset"
  | "key"
  | "user"
  | "members"
  | "external";

// Minimal HTMX type — we use a single method.
type HtmxApi = {
  ajax: (
    method: string,
    url: string,
    target?: string | HTMLElement | { target?: string; swap?: string },
  ) => Promise<void> | void;
};

declare global {
  interface Window {
    htmx?: HtmxApi;
  }
}

const ICONS: Record<IconKey, string> = {
  edit: `<path d="M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5z"/>`,
  delete: `<path d="M3 6h18"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>`,
  reset: `<circle cx="7.5" cy="15.5" r="5.5"/><path d="m21 2-9.6 9.6"/><path d="m15.5 7.5 3 3L22 7l-3-3"/>`,
  key: `<circle cx="7.5" cy="15.5" r="5.5"/><path d="m21 2-9.6 9.6"/><path d="m15.5 7.5 3 3L22 7l-3-3"/>`,
  user: `<path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/>`,
  members: `<path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M22 21v-2a4 4 0 0 0-3-3.87"/><path d="M16 3.13a4 4 0 0 1 0 7.75"/>`,
  external: `<path d="M15 3h6v6"/><path d="M10 14 21 3"/><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/>`,
};

function Icon({ name }: { name: IconKey }) {
  return (
    <svg
      width="14"
      height="14"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="1.5"
      stroke-linecap="round"
      stroke-linejoin="round"
      class="shrink-0"
      dangerouslySetInnerHTML={{ __html: ICONS[name] }}
    />
  );
}

type MenuState = {
  config: DropdownConfig;
  anchor: HTMLElement;
};

function Menu({
  state,
  onClose,
}: {
  state: MenuState | null;
  onClose: () => void;
}) {
  const menuRef = useRef<HTMLDivElement | null>(null);
  const [pos, setPos] = useState<{ top: number; left: number } | null>(null);

  useLayoutEffect(() => {
    if (!state) {
      setPos(null);
      return;
    }
    const rect = state.anchor.getBoundingClientRect();
    const align = state.config.align ?? "right";
    const menuEl = menuRef.current;
    const menuWidth = menuEl?.offsetWidth ?? 200;
    const menuHeight = menuEl?.offsetHeight ?? 0;

    let left =
      align === "right" ? rect.right - menuWidth : rect.left;
    let top = rect.bottom + 4;

    // Clamp inside viewport.
    const margin = 8;
    if (left + menuWidth > window.innerWidth - margin) {
      left = window.innerWidth - menuWidth - margin;
    }
    if (left < margin) left = margin;
    if (top + menuHeight > window.innerHeight - margin) {
      // Flip above the anchor.
      top = rect.top - menuHeight - 4;
      if (top < margin) top = margin;
    }
    setPos({ top, left });
  }, [state]);

  useEffect(() => {
    if (!state) return;
    const onDocClick = (e: MouseEvent) => {
      const t = e.target as Node | null;
      if (!t) return;
      if (menuRef.current && menuRef.current.contains(t)) return;
      if (state.anchor.contains(t)) return;
      onClose();
    };
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    document.addEventListener("mousedown", onDocClick);
    document.addEventListener("keydown", onKey);
    return () => {
      document.removeEventListener("mousedown", onDocClick);
      document.removeEventListener("keydown", onKey);
    };
  }, [state, onClose]);

  if (!state) return null;

  const itemBase =
    "flex items-center gap-2.5 w-full px-3 py-2 text-sm text-left transition-colors cursor-pointer border-0 bg-transparent no-underline";
  const itemNormal = "text-secondary hover:bg-hover hover:text-primary";
  const itemDanger = "text-danger hover:bg-danger-soft";

  const handleHtmxClick = (item: HtmxItem) => {
    onClose();
    if (item.hxConfirm && !window.confirm(item.hxConfirm)) return;
    const htmx = window.htmx;
    if (!htmx) {
      console.warn("htmx is not loaded; cannot invoke HTMX action");
      return;
    }
    const method = item.hxGet ? "GET" : item.hxPost ? "POST" : null;
    const url = item.hxGet ?? item.hxPost;
    if (!method || !url) return;
    htmx.ajax(method, url, {
      target: item.hxTarget ?? "#overlay-slot",
      swap: item.hxSwap ?? "innerHTML",
    });
  };

  return (
    <div
      ref={menuRef}
      role="menu"
      class="fixed z-50 min-w-44 bg-surface border border-default rounded shadow-elevated py-1"
      style={{
        top: pos?.top ?? -9999,
        left: pos?.left ?? -9999,
        visibility: pos ? "visible" : "hidden",
      }}
    >
      {state.config.items.map((item, idx) => {
        if (item.kind === "divider") {
          return (
            <div key={idx} class="h-px bg-subtle my-1" role="separator" />
          );
        }
        const cls = `${itemBase} ${item.danger ? itemDanger : itemNormal}`;
        if (item.kind === "link") {
          return (
            <a
              key={idx}
              href={item.href}
              class={cls}
              role="menuitem"
              onClick={() => onClose()}
            >
              {item.icon && <Icon name={item.icon} />}
              <span>{item.label}</span>
            </a>
          );
        }
        return (
          <button
            key={idx}
            type="button"
            class={cls}
            role="menuitem"
            onClick={() => handleHtmxClick(item)}
          >
            {item.icon && <Icon name={item.icon} />}
            <span>{item.label}</span>
          </button>
        );
      })}
    </div>
  );
}

function MenuRoot() {
  const [state, setState] = useState<MenuState | null>(null);

  useEffect(() => {
    const onClick = (e: MouseEvent) => {
      const target = (e.target as HTMLElement | null)?.closest<HTMLElement>(
        "[data-dropdown]",
      );
      if (!target) return;
      e.preventDefault();
      e.stopPropagation();
      const raw = target.getAttribute("data-dropdown");
      if (!raw) return;
      let config: DropdownConfig;
      try {
        config = JSON.parse(raw) as DropdownConfig;
      } catch (err) {
        console.warn("invalid data-dropdown JSON", err);
        return;
      }
      setState((prev) => {
        if (prev && prev.anchor === target) return null;
        return { config, anchor: target };
      });
    };
    document.addEventListener("click", onClick);
    return () => document.removeEventListener("click", onClick);
  }, []);

  return <Menu state={state} onClose={() => setState(null)} />;
}

export function mountDropdowns() {
  let host = document.getElementById("dropdown-root");
  if (!host) {
    host = document.createElement("div");
    host.id = "dropdown-root";
    document.body.appendChild(host);
  }
  render(h(MenuRoot, {}), host);
}
