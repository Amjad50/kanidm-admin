## Hard styling rule (Tailwind v4 + shadcn/tweakcn tokens)

All visual properties come from shadcn-shaped design tokens defined in
`styles/app.css` (so the theme can be tweaked verbatim with tweakcn online).
Light theme is `:root`; dark theme is activated by the `.dark` class on `<html>`.
Use only:

Baseline shadcn:

- `bg-background`, `text-foreground` (page chrome / body)
- `bg-card`, `bg-card-foreground` (surfaces / cards)
- `bg-popover`, `bg-popover-foreground` (elevated surfaces, menus, modals)
- `bg-accent`, `text-accent-foreground` (subtle hover/active surface — NOT the orange action color)
- `bg-primary`, `text-primary`, `text-primary-foreground` (the orange action color and its variants `bg-primary/90`, `bg-primary/80`)
- `bg-secondary`, `text-secondary-foreground` (secondary surface)
- `bg-muted`, `text-muted-foreground` (muted surface / muted text — use `text-muted-foreground/60` for disabled)
- `bg-destructive`, `text-destructive`, `border-destructive`, `text-destructive-foreground`
- `border-border`, `border-input` (single + slightly stronger border)
- `ring-ring`, `outline-ring/50` (focus rings)

Extensions (kept on top of shadcn baseline):

- `bg-primary-soft` (low-alpha primary background)
- `text-link`, `bg-link-soft`
- `bg-success`, `text-success`, `border-success`, `bg-success-soft`, `text-success-foreground`
- `bg-warning`, `text-warning`, `border-warning`, `bg-warning-soft`, `text-warning-foreground`
- `bg-destructive-soft`
- `bg-info`, `text-info`, `border-info`, `bg-info-soft`, `text-info-foreground`
- `bg-code-bg`, `bg-token-bg`, `text-mono-chip`
- `accent-primary` (CSS `accent-color` utility, for native form controls)
- `shadow-primary-ring` (subtle ring around primary-color dots)

Radius (driven by `--radius`):

- `rounded-sm` (`--radius` − 4px)
- `rounded-md` (`--radius` − 2px)
- `rounded-lg` (`--radius`)
- `rounded-xl` (`--radius` + 4px)
- `rounded-pill` (999px — extension)

Shadows: `shadow-sm`, `shadow`, `shadow-md`, `shadow-lg`, `shadow-xl`, `shadow-2xl`.

Fonts: `font-sans`, `font-mono`.

NEVER use:

- Raw Tailwind palette colors: `bg-zinc-900`, `text-gray-500`, etc.
- Raw hex values in `style="..."` attributes
- `bg-(--var)` arbitrary-value escape hatches
- Inline color declarations
- The OLD vocabulary (`bg-canvas`, `bg-surface`, `bg-elevated`, `bg-hover`,
  `bg-active`, `text-tertiary`, `text-disabled`, `border-subtle`,
  `border-default`, `border-strong`, `bg-danger`, `shadow-card`, etc.) —
  all migrated, do not reintroduce.

If a template needs a color or radius not in app.css, ADD IT to app.css
as a new var on both `:root` and `.dark` and mirror it in `@theme inline` —
do not work around it.

Default to using Bun instead of Node.js.

- Use `bun <file>` instead of `node <file>` or `ts-node <file>`
- Use `bun test` instead of `jest` or `vitest`
- Use `bun build <file.html|file.ts|file.css>` instead of `webpack` or `esbuild`
- Use `bun install` instead of `npm install` or `yarn install` or `pnpm install`
- Use `bun run <script>` instead of `npm run <script>` or `yarn run <script>` or `pnpm run <script>`
- Use `bunx <package> <command>` instead of `npx <package> <command>`
- Bun automatically loads .env, so don't use dotenv.

## APIs

- `Bun.serve()` supports WebSockets, HTTPS, and routes. Don't use `express`.
- `bun:sqlite` for SQLite. Don't use `better-sqlite3`.
- `Bun.redis` for Redis. Don't use `ioredis`.
- `Bun.sql` for Postgres. Don't use `pg` or `postgres.js`.
- `WebSocket` is built-in. Don't use `ws`.
- Prefer `Bun.file` over `node:fs`'s readFile/writeFile
- Bun.$`ls` instead of execa.

## Testing

Use `bun test` to run tests.

```ts#index.test.ts
import { test, expect } from "bun:test";

test("hello world", () => {
  expect(1).toBe(1);
});
```

## Frontend

Use HTML imports with `Bun.serve()`. Don't use `vite`. HTML imports fully support React, CSS, Tailwind.

Server:

```ts#index.ts
import index from "./index.html"

Bun.serve({
  routes: {
    "/": index,
    "/api/users/:id": {
      GET: (req) => {
        return new Response(JSON.stringify({ id: req.params.id }));
      },
    },
  },
  // optional websocket support
  websocket: {
    open: (ws) => {
      ws.send("Hello, world!");
    },
    message: (ws, message) => {
      ws.send(message);
    },
    close: (ws) => {
      // handle close
    }
  },
  development: {
    hmr: true,
    console: true,
  }
})
```

HTML files can import .tsx, .jsx or .js files directly and Bun's bundler will transpile & bundle automatically. `<link>` tags can point to stylesheets and Bun's CSS bundler will bundle.

```html#index.html
<html>
  <body>
    <h1>Hello, world!</h1>
    <script type="module" src="./frontend.tsx"></script>
  </body>
</html>
```

With the following `frontend.tsx`:

```tsx#frontend.tsx
import React from "react";
import { createRoot } from "react-dom/client";

// import .css files directly and it works
import './index.css';

const root = createRoot(document.body);

export default function Frontend() {
  return <h1>Hello, world!</h1>;
}

root.render(<Frontend />);
```

Then, run index.ts

```sh
bun --hot ./index.ts
```

For more information, read the Bun API docs in `node_modules/bun-types/docs/**.mdx`.
