# 92 — Loading States (Cross-Cutting Pattern)

How async loading is communicated across the UI: skeleton screens, spinners, suspense boundaries, progressive rendering.

## Purpose

Never show a blank screen while data is loading. Indicate progress with the right type of loading affordance for the right context. Avoid layout shift when data arrives.

## Loading affordances

### 1. Skeleton screens

For initial loads of structured content (tables, detail pages, cards).

**Appearance:**
- Same shape as the eventual content: rectangular blocks roughly matching text widths and heights
- Background: `--bg-hover` or a slightly elevated surface
- Subtle shimmer or pulse animation (per design system motion spec; respect reduce-motion)
- Replaces actual content during initial fetch

**When to use:**
- First load of a list (people, groups, OAuth2)
- First load of a detail page
- Tab content load when switching tabs that fetch data

**Examples:**
- People list: 6-8 skeleton rows with avatar circle + 2 text lines (name + SPN)
- Detail page: skeleton identity card + skeleton tab content
- Dashboard: skeleton metric cards with rectangular number placeholders

### 2. Spinners

For in-button or in-context async actions where the layout doesn't change.

**Appearance:**
- Lucide `Loader2` icon spinning, sized 14-16px (button) or 20-24px (card-level)
- Inherits text color or design system's accent

**When to use:**
- Submit button: "Save" → "Saving…" with spinner replacing icon
- Inline action: a row's "Delete" button showing spinner during delete
- Card-level refresh: small spinner in the corner of a card while refetching

### 3. Suspense boundaries

For larger sections of a page that load independently.

**Appearance:**
- A wrapped section transitions from "loading state" (skeleton or spinner) to "content state" smoothly
- Multiple suspense boundaries on one page can resolve at different times — e.g., the identity card loads first, then the tabs load their data separately

**When to use:**
- Dashboard: each metric card can resolve independently
- Detail page: identity card resolves first, tabs lazy-load their data
- Lists with optional related data (e.g., a person's group memberships fetched separately from their core data)

### 4. Page-level full spinner

For the rare case where there's nothing to skeleton (e.g., navigation between unrelated pages without prefetching).

**Appearance:**
- Centered spinner, no text or with a single "Loading…" label
- Used only for very brief moments

**When to use:**
- First load after sign-in, before the dashboard skeleton renders
- Rare; skeletons are almost always better

### 5. Progress bars

For multi-step operations with known progress.

**Appearance:**
- Linear progress bar, semantic color
- Used in bulk operations: "Deleting 5 people…", showing 2/5 done

**When to use:**
- Bulk delete (5 people, 5 groups, 5 OAuth2 apps)
- Bulk session destroy
- File upload (image upload for OAuth2 image)

### 6. Optimistic updates

For actions where the result is predictable (most CRUD operations).

**Appearance:**
- Updates the UI immediately, before the server confirms
- If the server rejects, the UI rolls back and shows an error

**When to use:**
- Adding/removing group members (immediate visual change)
- Toggling OAuth2 settings (the toggle flips immediately, server-write happens in background; rollback on error)
- Star/unstar primary email
- Deleting a row from a list (row fades out immediately; rollback on error with toast)

Risky for irreversible operations — don't optimistically delete entire entities. Do optimistically reorder mail addresses or add a member.

## Layout shift prevention

- Skeleton dimensions should match real content dimensions as closely as possible
- Reserve space for things that haven't loaded yet (e.g., reserve image dimensions in OAuth2 cards even before the image URL is known)
- Use CSS `aspect-ratio` and `min-height` to prevent jumps

## Timing thresholds

- **< 100ms response:** no loading affordance needed; content just appears
- **100-1000ms:** skeleton or spinner appears immediately
- **1-3 seconds:** keep showing the loading state; consider showing a small "Still loading…" hint after 2s
- **> 3 seconds:** include a cancel/retry option if possible
- **Indefinite (e.g., waiting for WebAuthn):** dedicated waiting screen (e.g., screen 06)

## Sample data references

For populated content that the skeletons are placeholders for:
- Use Alice Smith and the other sample people for the People list skeletons
- Use Grafana etc. for OAuth2 list skeletons

## Design system variations

- **Linear:** subtle, fast skeleton shimmer (1.5s cycle); tight skeletons matching dense rows
- **Cloudflare:** comfortable skeleton, slightly slower shimmer (1.8s)
- **Stripe:** softest treatment — pulse rather than shimmer; longer cycle (2s); the skeletons themselves can be slightly more elaborate (rounded corners, layered subtle gradients)

## Reduce-motion considerations

- Disable shimmer / pulse animation
- Replace with static `--bg-hover` blocks
- Spinner can still rotate (it's the indicator, not decoration), but at a slower speed if user prefers reduced motion

## Mockup elements to render

Render 3-4 distinct loading state mockups:

1. **Table skeleton** — People list with 6 skeleton rows showing avatar + name + SPN + status placeholders
2. **Detail page skeleton** — Person detail with skeleton identity card + skeleton tab bar + skeleton overview content
3. **Button spinner** — A primary button showing "Saving…" with Loader2 icon
4. **Page-level full spinner** — small mockup of a centered spinner with "Loading…" label
5. (Optional) Bulk progress — "Deleting 3 of 5 people…" with a linear progress bar
