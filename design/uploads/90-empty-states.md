# 90 — Empty States (Cross-Cutting Pattern)

Visual treatment for "nothing here yet" states across the admin UI. One consistent pattern used in many places.

## Purpose

Every list, table, or section that can be empty needs a thoughtful empty state. The goal: tell the user clearly what they're looking at, suggest the next action, and avoid a blank rectangle that feels broken.

## The pattern

A centered empty state has three to four elements:

1. **Illustration / icon** (optional, small) — keep it minimal. A line-art icon or a small abstract shape in the design system's `--text-tertiary` color. No mascots, no characters, no whimsy.
2. **Heading** — short, descriptive: "No people yet" / "No OAuth2 applications" / "No SSH keys registered" — never "Oops!" or "Nothing to see here!"
3. **Body text** — one or two sentences. Explain what would go here and how to add to it. e.g., "Create your first OAuth2 application to enable SSO for one of your services."
4. **Primary CTA** — a button that performs the most likely next action: "+ Create person", "+ Add SSH key", etc. Sometimes omitted for sub-sections where the action is elsewhere.

## Variants by context

### Empty list pages (after first load, no entries exist)

A larger empty state, page-centered. Vertical layout:
- Icon (32-40px) — domain-appropriate Lucide icon (Users for people, UsersRound for groups, Shield for OAuth2)
- Heading (font-lg or font-xl, font-semibold)
- Body (font-base, `--text-secondary`)
- Primary CTA button

Examples:

**People list — no people:**
- Icon: Users
- Heading: "No people yet"
- Body: "Create your first person to get started with kanidm."
- CTA: "+ Create person"

**Groups list — no groups (rare, since kanidm ships built-ins):**
- Icon: UsersRound
- Heading: "No groups yet"
- Body: "Create a group to organize people and configure access policies."
- CTA: "+ Create group"

**OAuth2 list — no apps:**
- Icon: Shield
- Heading: "No OAuth2 applications"
- Body: "Create your first OAuth2 application to enable SSO for one of your services like Grafana, Nextcloud, or Gitea."
- CTA: "+ Create OAuth2 application"

### Filtered-empty (list has entries, but filter / search yielded zero)

Smaller empty state inside the table area:
- No or very subtle icon
- Heading: "No people match '{query}'" / "No groups match the current filters"
- CTA: "Clear search" / "Clear filters" (text link)

No "create" CTA here — the user is in search mode, not creation mode.

### Empty list sections within a detail page

(e.g., "Members" section of a group with 0 members, "SSH Keys" section with no keys.)

Inline, compact:
- Single line of text in `--text-secondary`: "No members yet."
- Inline action button: "+ Add members"
- No icon (the surrounding context provides the visual frame)

Or even more compact for sub-sub-sections (e.g., a 4-line "Recent activity" card with no activity):
- Just text "—" or "None" in `--text-tertiary`

### Empty after a destructive action

When the admin successfully purges or destroys all members of something (e.g., purged all members of a group), the same empty state appears immediately — no special "you just deleted everything" framing. The toast that confirms the action provides the context.

### Empty sessions

For a person with no active sessions:
- "No active sessions."
- Subdued helper text: "{Person} is not currently signed in to any device."

### Empty members of a group

- "No members yet."
- "+ Add members" button

### Empty SSH keys / scope maps / claim maps

- "No SSH keys registered." / "No scope maps configured."
- Action button: "+ Add SSH key" / "+ Add scope map"

### Empty mail list

- "No emails set."
- Inline button: "+ Add email"

### Empty signing keys

- This shouldn't happen — kanidm always has at least one signing key for an OAuth2 client.
- If it does (unrecoverable state), show: "No signing keys. → Schedule a rotation now to create one."

## Visual treatment

- Icon size: scale to context. Page-level empty: 32-40px. Section empty: 20-24px or none.
- Icon color: `--text-tertiary` for muted feel.
- Heading color: `--text-primary`.
- Body color: `--text-secondary`.
- CTA: primary button. Width matches button conventions per design system (not full-width).
- Centered vertically and horizontally within the parent container.
- Generous whitespace above and below (per design system spacing).

For the **Linear variant**: minimal, dense empty state. Maybe just text + button, no icon.
For the **Cloudflare variant**: comfortable, friendly. Icon + heading + body + CTA. Maybe a soft tinted background for the icon container.
For the **Stripe variant**: roomy, with possibly a softer gradient backdrop behind the icon. The empty state itself is a moment of polish, not a void.

## Sample data references

Examples to render:
- "No people yet" — `_sample-data.md` provides sample people for the populated state
- "No SSH keys registered" — for Jane Doe (newly-created person)
- "No active sessions" — for Bob Jones (not currently signed in)
- "No claim maps configured" — for Grafana (which has scope maps but no claim maps)

## Tone

Direct, encouraging without being saccharine. "Get started by creating…" is fine. "Looks like you haven't created anyone yet! 🎉" is not. The product is an identity management server; the tone is professional throughout.

## Mockup elements to render

- Render 3 distinct empty state variants:
  1. **Page-level empty** (e.g., the People list with zero people): icon + heading "No people yet" + body + "+ Create person" CTA, centered in the main content area
  2. **Filtered empty** (e.g., search returned nothing): smaller, inline within the table area, "No people match 'xyzabc'" + Clear search link
  3. **Section empty** (e.g., the Members tab of a freshly-created group): compact text + inline action button "+ Add members"
