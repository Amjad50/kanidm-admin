// Behavior registry — delegated DOM enhancement.
//
// A behavior is a (selector, event, handler) triple registered at module load
// via defineBehavior(). One document-level listener per event dispatches to
// all behaviors registered for that event, using .closest(selector) to find
// the marker element. Behaviors must be idempotent — they're stateless at the
// JS level, and HTMX swaps re-attach nothing because listeners are on
// document.

type EventName = 'click' | 'auxclick' | 'input' | 'change';

interface Behavior {
  selector: string;
  event: EventName;
  handler: (target: HTMLElement, event: Event) => void;
}

const registered: Behavior[] = [];

export function defineBehavior(b: Behavior): void {
  registered.push(b);
}

export function mountBehaviors(): void {
  const byEvent = new Map<EventName, Behavior[]>();
  for (const b of registered) {
    if (!byEvent.has(b.event)) byEvent.set(b.event, []);
    byEvent.get(b.event)!.push(b);
  }
  for (const [event, list] of byEvent) {
    document.addEventListener(event, (e) => {
      const root = e.target as HTMLElement | null;
      if (!root) return;
      for (const b of list) {
        const match = root.closest<HTMLElement>(b.selector);
        if (match) b.handler(match, e);
      }
    });
  }
}
