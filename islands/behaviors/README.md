# Behaviors

Delegated DOM enhancement. One file per behavior. Registered via
`defineBehavior({ selector, event, handler })`. Mounted once by
`mountBehaviors()` in `entry.ts`.

## When to add a behavior

Add a behavior when:

- A marker `data-*` attribute on an element causes simple DOM mutation, a
  `window.*` call, or HTMX-adjacent wiring.
- The same enhancement is needed on multiple elements / multiple pages.
- The behavior is stateless at the JS level (state lives in the DOM, in
  `localStorage`, or in `clipboard`).

## When NOT to add a behavior

- You need component state that survives across renders, or coordinate several
  elements via shared state → write a Preact island under `islands/<name>.tsx`.
- You want logic in an HTML attribute (e.g. `onclick="if(...) return"`) →
  write a behavior. Behaviors exist *because* JS-in-attribute is bad.

## Contract

Each behavior file does this once at module-load:

```ts
import { defineBehavior } from './index';

defineBehavior({
  selector: '[data-thing]',
  event: 'click',
  handler: (el, event) => { /* ... */ },
});
```

Then `entry.ts` imports the file (side-effect import) and calls
`mountBehaviors()` once. HTMX swaps don't need to re-mount because listeners
are on `document`.
