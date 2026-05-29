// Pagination control rendered by Preact, with HTMX driving the row swap.
//
// Server-side: emit a single empty <div> with the data attributes describing
// the list state. This script finds every such container at mount time and
// renders a real <Pagination> into it. Subsequent page clicks update the
// component's own state AND fire an HTMX request to swap the rows region.
//
// Container contract:
//   <div data-pagination
//        data-page="3"
//        data-total-pages="12"
//        data-filtered-count="234"
//        data-base-url="/people"
//        data-target="#people-tbody"></div>
//
// The component owns "current page". Server-rendered HTML never has to
// re-emit the pagination markup; only the rows fragment is returned by the
// HTMX endpoint.

import { h, render } from "preact";
import { useEffect, useState } from "preact/hooks";

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

type PageItem = { kind: "page"; n: number } | { kind: "ellipsis" };

function windowed(current: number, total: number): PageItem[] {
  if (total <= 1) return [];
  if (total <= 7) {
    return Array.from({ length: total }, (_, i) => ({
      kind: "page" as const,
      n: i + 1,
    }));
  }
  const items: PageItem[] = [{ kind: "page", n: 1 }];
  const windowStart = Math.max(2, current - 1);
  const windowEnd = Math.min(total - 1, current + 1);
  if (windowStart > 2) items.push({ kind: "ellipsis" });
  for (let p = windowStart; p <= windowEnd; p++) {
    items.push({ kind: "page", n: p });
  }
  if (windowEnd < total - 1) items.push({ kind: "ellipsis" });
  items.push({ kind: "page", n: total });
  return items;
}

type Props = {
  initialPage: number;
  totalPages: number;
  filteredCount: number;
  perPage: number;
  baseUrl: string;
  target: string;
};

function Pagination({
  initialPage,
  totalPages,
  filteredCount,
  perPage,
  baseUrl,
  target,
}: Props) {
  const [page, setPage] = useState(initialPage);

  // When HTMX swaps the rows region (search, filter, or our own page click),
  // re-read `?page=N` from the URL. hx-push-url keeps the URL authoritative
  // and a search reset may have changed which page is current.
  useEffect(() => {
    const handler = (evt: Event) => {
      const detail = (evt as CustomEvent).detail as
        | { target?: HTMLElement }
        | undefined;
      if (!detail?.target) return;
      const targetEl = document.querySelector(target);
      if (!targetEl) return;
      if (
        !detail.target.contains(targetEl as Node) &&
        detail.target !== targetEl
      ) {
        return;
      }
      const url = new URL(window.location.href);
      const pageParam = url.searchParams.get("page");
      const n = pageParam ? parseInt(pageParam, 10) : 1;
      if (!isNaN(n) && n >= 1) setPage(n);
    };
    document.body.addEventListener("htmx:afterSwap", handler);
    return () => document.body.removeEventListener("htmx:afterSwap", handler);
  }, [target]);

  if (totalPages <= 1) return null;

  const pageStart = filteredCount === 0 ? 0 : (page - 1) * perPage + 1;
  const pageEnd = Math.min(filteredCount, page * perPage);

  const go = (n: number) => {
    if (n < 1 || n > totalPages || n === page) return;
    const htmx = window.htmx;
    if (!htmx) {
      console.warn("htmx not available; navigating instead");
      window.location.href = `${baseUrl}?page=${n}`;
      return;
    }
    // Build the URL from the current location so we preserve filter params
    // (q=, status=, etc.) — same behavior as the old hx-include="closest form".
    const url = new URL(window.location.href);
    url.searchParams.set("page", String(n));
    // Push history first so the afterSwap handler can read the new ?page=.
    window.history.pushState({}, "", url.toString());
    setPage(n);
    // Strip origin — htmx.ajax wants a same-origin path.
    const path = url.pathname + url.search;
    htmx.ajax("GET", path, { target, swap: "innerHTML" });
  };

  const items = windowed(page, totalPages);

  const btnBase =
    "w-8 h-8 rounded border text-sm inline-flex items-center justify-center transition-colors";
  const btnInactive =
    "border-default bg-surface text-secondary hover:bg-hover hover:text-primary";
  const btnActive = "border-accent bg-accent text-on-accent";
  const btnDisabled =
    "border-default bg-surface text-secondary disabled:opacity-50 disabled:cursor-not-allowed";

  return (
    <div class="flex items-center justify-between mt-4 text-sm text-secondary">
      <div>
        Showing
        <span class="text-primary font-medium">
          {" "}
          {pageStart}–{pageEnd}{" "}
        </span>
        of
        <span class="text-primary font-medium"> {filteredCount}</span>
      </div>
      <div class="flex items-center gap-1">
        <button
          type="button"
          aria-label="Previous page"
          class={`${btnBase} ${page > 1 ? btnInactive : btnDisabled}`}
          disabled={page <= 1}
          onClick={() => go(page - 1)}
        >
          <svg
            width="14"
            height="14"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="1.5"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <path d="m15 18-6-6 6-6" />
          </svg>
        </button>

        {items.map((item, idx) =>
          item.kind === "ellipsis" ? (
            <span
              key={`e${idx}`}
              class="w-8 h-8 text-sm inline-flex items-center justify-center text-tertiary"
              aria-hidden="true"
            >
              …
            </span>
          ) : item.n === page ? (
            <button
              key={item.n}
              type="button"
              aria-current="page"
              class={`${btnBase} ${btnActive}`}
            >
              {item.n}
            </button>
          ) : (
            <button
              key={item.n}
              type="button"
              class={`${btnBase} ${btnInactive}`}
              onClick={() => go(item.n)}
            >
              {item.n}
            </button>
          ),
        )}

        <button
          type="button"
          aria-label="Next page"
          class={`${btnBase} ${page < totalPages ? btnInactive : btnDisabled}`}
          disabled={page >= totalPages}
          onClick={() => go(page + 1)}
        >
          <svg
            width="14"
            height="14"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="1.5"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <path d="m9 18 6-6-6-6" />
          </svg>
        </button>
      </div>
    </div>
  );
}

// Marker we set on a host once we've rendered into it, so subsequent calls
// (e.g., after an HTMX swap of the rows region triggers a re-mount) skip it.
const MOUNTED_MARK = "data-pagination-mounted";

function mountOne(host: HTMLElement) {
  if (host.getAttribute(MOUNTED_MARK)) return;
  const page = parseInt(host.dataset.page ?? "1", 10) || 1;
  const totalPages = parseInt(host.dataset.totalPages ?? "1", 10) || 1;
  const filteredCount = parseInt(host.dataset.filteredCount ?? "0", 10) || 0;
  const perPage = parseInt(host.dataset.perPage ?? "15", 10) || 15;
  const baseUrl = host.dataset.baseUrl ?? "";
  const target = host.dataset.target ?? "";
  if (!baseUrl || !target) {
    console.warn(
      "[data-pagination] missing baseUrl or target; skipping mount",
      host,
    );
    return;
  }
  host.setAttribute(MOUNTED_MARK, "1");
  render(
    h(Pagination, {
      initialPage: page,
      totalPages,
      filteredCount,
      perPage,
      baseUrl,
      target,
    }),
    host,
  );
}

export function mountPagination() {
  const hosts = document.querySelectorAll<HTMLElement>("[data-pagination]");
  hosts.forEach(mountOne);

  // After any HTMX swap, the server may have replaced our host (e.g., the
  // pagination data was updated via OOB swap because total_pages changed).
  // Bind any newly-arrived hosts.
  document.body.addEventListener("htmx:afterSettle", () => {
    document.querySelectorAll<HTMLElement>("[data-pagination]").forEach(mountOne);
  });
}
