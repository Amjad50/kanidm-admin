import { render } from "preact";
import { useEffect, useRef, useState } from "preact/hooks";
import { AlertTriangle, CircleCheck, CircleX, Info, X as XIcon } from "lucide-preact";

export type ToastKind = "success" | "info" | "warn" | "error";

export type ToastPayload = {
  title: string;
  desc?: string;
  kind?: ToastKind;
};

type Toast = ToastPayload & { id: number; kind: ToastKind };

const DURATION_MS = 5000;
const EXIT_MS = 180;
let nextId = 1;

const subscribers = new Set<(t: Toast) => void>();

function emit(payload: ToastPayload) {
  const toast: Toast = { id: nextId++, kind: payload.kind ?? "info", ...payload };
  for (const fn of subscribers) fn(toast);
}

function ToastStack() {
  // newest first → stack top-down with newest on top
  const [toasts, setToasts] = useState<Toast[]>([]);
  const [exiting, setExiting] = useState<Set<number>>(new Set());

  useEffect(() => {
    const sub = (t: Toast) => setToasts((prev) => [t, ...prev]);
    subscribers.add(sub);
    return () => {
      subscribers.delete(sub);
    };
  }, []);

  const dismiss = (id: number) => {
    setExiting((prev) => new Set(prev).add(id));
    setTimeout(() => {
      setToasts((prev) => prev.filter((t) => t.id !== id));
      setExiting((prev) => {
        const next = new Set(prev);
        next.delete(id);
        return next;
      });
    }, EXIT_MS);
  };

  return (
    <div class="fixed top-4 right-4 flex flex-col gap-2.5 z-50">
      {toasts.map((t) => (
        <ToastItem
          key={t.id}
          toast={t}
          exiting={exiting.has(t.id)}
          onDismiss={() => dismiss(t.id)}
        />
      ))}
    </div>
  );
}

function ToastItem({
  toast,
  exiting,
  onDismiss,
}: {
  toast: Toast;
  exiting: boolean;
  onDismiss: () => void;
}) {
  const [entered, setEntered] = useState(false);
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    // Animate in on next frame so the transition runs.
    const raf = requestAnimationFrame(() => setEntered(true));
    timerRef.current = setTimeout(onDismiss, DURATION_MS);
    return () => {
      cancelAnimationFrame(raf);
      if (timerRef.current) clearTimeout(timerRef.current);
    };
  }, []);

  const borderColor: Record<ToastKind, string> = {
    success: "border-l-success",
    info: "border-l-info",
    warn: "border-l-warning",
    error: "border-l-danger",
  };
  const iconColor: Record<ToastKind, string> = {
    success: "text-success",
    info: "text-info",
    warn: "text-warning",
    error: "text-danger",
  };

  const stateClasses = exiting
    ? "opacity-0 translate-x-2"
    : entered
      ? "opacity-100 translate-x-0"
      : "opacity-0 translate-x-4";

  return (
    <div
      role="status"
      class={`bg-elevated border border-subtle border-l-4 ${borderColor[toast.kind]} rounded-md shadow-elevated w-[360px] px-4 py-3.5 flex items-start gap-3 transition-all duration-200 motion-reduce:transition-opacity motion-reduce:duration-100 motion-reduce:translate-x-0 ${stateClasses}`}
    >
      <span class={`shrink-0 mt-0.5 ${iconColor[toast.kind]}`}>
        <KindIcon kind={toast.kind} />
      </span>
      <div class="flex-1 min-w-0">
        <div class="text-sm font-medium text-primary leading-snug">{toast.title}</div>
        {toast.desc && (
          <div class="text-[13px] text-secondary mt-0.5 leading-relaxed">{toast.desc}</div>
        )}
      </div>
      <button
        type="button"
        onClick={onDismiss}
        aria-label="Dismiss"
        class="shrink-0 -mr-1 -mt-0.5 w-6 h-6 rounded-sm inline-flex items-center justify-center text-tertiary hover:text-primary hover:bg-hover transition-colors cursor-pointer border-0 bg-transparent"
      >
        <XIcon size={14} class="shrink-0" />
      </button>
    </div>
  );
}

function KindIcon({ kind }: { kind: ToastKind }) {
  switch (kind) {
    case "success":
      return <CircleCheck size={18} class="shrink-0" />;
    case "warn":
      return <AlertTriangle size={18} class="shrink-0" />;
    case "error":
      return <CircleX size={18} class="shrink-0" />;
    case "info":
    default:
      return <Info size={18} class="shrink-0" />;
  }
}

export function mountToasts() {
  const host = document.getElementById("toast-stack");
  if (!host) return;

  render(<ToastStack />, host);

  document.body.addEventListener("toast", (event) => {
    const detail = (event as CustomEvent).detail as ToastPayload | ToastPayload[] | undefined;
    if (!detail) return;
    const items = Array.isArray(detail) ? detail : [detail];
    for (const item of items) emit(item);
  });

  (window as unknown as { showToast?: (t: ToastPayload) => void }).showToast = emit;
}
