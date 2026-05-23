# Artifex Design System

## Overview

Artifex is a desktop AI studio for game developers. The visual identity is **dark, technical, and focused** — a deep navy canvas with vibrant coral-red accents that evoke creative energy against a professional workspace. The aesthetic sits between a game engine IDE and a creative tool: dense but not cluttered, powerful but approachable.

**Personality**: Professional gaming, dark-mode-first, creative-tool energy.
**Mood**: Focused, immersive, techy — like a cockpit for game asset creation.
**Color mode**: Dark only (no light theme).
**Stack**: Tauri v2 + SvelteKit 2 + Tailwind CSS 4.

---

## Colors

### Primary Palette

| Token | Hex | Role | Usage |
|-------|-----|------|-------|
| `--color-canvas` | `#1a1a2e` | Background | Root background, main workspace area, input fields |
| `--color-panel` | `#16213e` | Surface | Sidebar, status bar, dialogs, cards, sections, properties panel |
| `--color-surface` | `#0f3460` | Elevated / Border | Borders (sidebar, panels, dialogs, inputs), hover backgrounds, badges container, secondary buttons |
| `--color-accent` | `#e94560` | Accent / CTA | Primary buttons, selected item ring, logo, focus borders, required markers, active indicators |
| `--color-text` | `#eaeaea` | Primary text | Headings, body text, labels, card titles, button text |
| `--color-text-muted` | `#9ca3af` | Secondary text | Descriptions, metadata, timestamps, placeholder text, disabled states, icons in empty states |

### Semantic Colors

| Color | Hex | Role | Usage |
|-------|-----|------|-------|
| Success / Active | `green-500` | Status | Active project badge (`bg-green-500/20 text-green-400`), "✓ Asset generated" |
| Warning / Archived | `yellow-500` | Status | Archived project badge (`bg-yellow-500/20 text-yellow-400`) |
| Error | `red-500` | Error | Error messages (`bg-red-500/20 border-red-500/50 text-red-400`), required markers |
| Info / Image kind | `blue-500` | Info | Image asset kind badge (`bg-blue-500/20 text-blue-400`) |
| Sprite kind | `green-500` | Kind | Sprite asset badges (`bg-green-500/20 text-green-400`) |
| Tileset kind | `purple-500` | Kind | Tileset badges (`bg-purple-500/20 text-purple-400`) |
| Material kind | `orange-500` | Kind | Material badges (`bg-orange-500/20 text-orange-400`) |
| Audio kind | `yellow-500` | Kind | Audio badges (`bg-yellow-500/20 text-yellow-400`) |
| Voice kind | `pink-500` | Kind | Voice badges (`bg-pink-500/20 text-pink-400`) |
| Video kind | `red-500` | Kind | Video badges (`bg-red-500/20 text-red-400`) |
| Pro tier | `purple-500` | Tier | Pro badge (`bg-purple-500/20 text-purple-400`) |
| Free tier | `gray-500` | Tier | Free badge (`bg-gray-500/20 text-gray-400`) |

### Opacity Patterns

All semantic badge colors use `/20` opacity for backgrounds (e.g., `bg-green-500/20`).
Accent buttons use `/80` on hover (e.g., `hover:bg-[var(--color-accent)]/80`).
Panels use `/80` on hover for cards (e.g., `hover:bg-[var(--color-panel)]/80`).
Error borders use `/50` (e.g., `border-red-500/50`).

---

## Typography

### Font Family

```
Primary: 'Inter', system-ui, -apple-system, sans-serif
Code:    monospace (system default via `font-mono`)
```

### Scale

| Level | Tailwind | Size | Weight | Usage |
|-------|----------|------|--------|-------|
| H1 | `text-2xl` | ~24px | `font-bold` | Page titles ("Projects") |
| H2 | `text-lg` | ~18px | `font-semibold` | Dialog titles, section headings ("Create New Project") |
| H3 | `text-lg` | ~18px | `font-semibold` | Card titles (project names) |
| Body | `text-base` | ~16px | `font-normal` | Default body text |
| Small | `text-sm` | ~14px | `font-normal` / `font-medium` | Card descriptions, input values, metadata |
| XS | `text-xs` | ~12px | `font-normal` / `font-medium` / `font-semibold` | Labels, timestamps, badges, kind tags, status text |
| Label | `text-xs` + `uppercase tracking-wider` | ~12px | `font-semibold` | Property panel labels ("NAME", "KIND", "STATUS") |

### Text Colors by Role

| Role | Color Token | Tailwind |
|------|-------------|----------|
| Primary text | `--color-text` | `text-[var(--color-text)]` |
| Muted/secondary text | `--color-text-muted` | `text-[var(--color-text-muted)]` |
| Accent text | `--color-accent` | `text-[var(--color-accent)]` |
| Required marker | red-400 | `text-red-400` |

---

## Layout

### App Shell

```
┌─────────────────────────────────────────────────┐
│ AppShell (h-screen w-screen flex flex-col)       │
│ ┌──────┬─────────────────────────┬─────────────┐│
│ │      │                         │             ││
│ │Side  │    Main Workspace       │  Properties ││
│ │bar   │    (flex-1)             │  Panel      ││
│ │w-64  │                         │  w-72       ││
│ │w-16  │                         │  w-12       ││
│ │(col) │                         │  (collapsed)││
│ └──────┴─────────────────────────┴─────────────┘│
│ ┌─────────────────────────────────────────────┐ │
│ │ Status Bar (h-8)                            │ │
│ └─────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────┘
```

### Sidebar

- **Expanded**: `w-64` (256px) — `bg-[var(--color-panel)]`, `border-r border-[var(--color-surface)]`
- **Collapsed**: `w-16` (64px) — icons only
- **Transition**: `transition-all duration-200`
- **Sections**: Header (logo + toggle) → Navigation → Recent Projects → New Project button
- **Section dividers**: `border-b border-[var(--color-surface)]`
- **Navigation items**: `px-3 py-2 rounded-lg`, active state: `bg-[var(--color-surface)]`, hover: `hover:bg-[var(--color-surface)]`

### Properties Panel

- **Expanded**: `w-72` (288px) — `bg-[var(--color-panel)]`, `border-l border-[var(--color-surface)]`
- **Collapsed**: `w-12` (48px)

### Status Bar

- **Height**: `h-8` (32px)
- **Background**: `bg-[var(--color-panel)]`, `border-t border-[var(--color-surface)]`
- **Text**: `text-xs text-[var(--color-text-muted)]`
- **Layout**: `flex justify-between px-4` — version/model | project name | job status

### Grid Layouts

- **Project cards**: `grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4`
- **Asset cards**: `grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-4`
- **Settings form**: `max-w-4xl`, `grid gap-4 md:grid-cols-2`

---

## Spacing

### Padding Scale (most used)

| Value | Usage |
|-------|-------|
| `p-1` / `p-1.5` | Icon button padding, toggle buttons |
| `p-2` | Small buttons, collapsed sidebar items |
| `p-3` | Asset cards body, list items |
| `p-4` | Section padding, sidebar sections, dialog body, page padding |
| `p-6` | Settings sections, page-level padding |

### Gap Scale

| Value | Usage |
|-------|-------|
| `gap-1` | Tight lists (sidebar projects) |
| `gap-2` | Form rows, inline button groups, icon + text |
| `gap-3` | Navigation items, inline elements |
| `gap-4` | Grid gaps, form sections, card grids |
| `gap-6` | Settings grid |

---

## Components

### Buttons

#### Primary Button (CTA)
```
px-4 py-2 bg-[var(--color-accent)] hover:bg-[var(--color-accent)]/80 
rounded-lg transition-colors font-medium
```
Use for: "Create Project", "Generate", primary actions.

#### Primary Button — Disabled
```
Same as primary + disabled:opacity-50
```

#### Secondary Button (Surface)
```
px-4 py-2 rounded-lg bg-[var(--color-surface)] hover:bg-[var(--color-surface)]/80 
transition-colors font-medium
```
Use for: "Browse", secondary actions.

#### Ghost Button
```
px-4 py-2 rounded-lg hover:bg-[var(--color-surface)] transition-colors
```
Use for: "Cancel", non-primary actions.

#### Icon Button
```
p-1.5 rounded hover:bg-[var(--color-surface)] transition-colors 
text-[var(--color-text-muted)]
```
Use for: Toggle buttons, close buttons, toolbar icons.

#### Full-width Button (Sidebar CTA)
```
w-full flex items-center justify-center gap-2 px-4 py-2 
bg-[var(--color-accent)] hover:bg-[var(--color-accent)]/80 
rounded-lg transition-colors font-medium
```

### Input Fields
```
w-full px-3 py-2 rounded-lg bg-[var(--color-canvas)] 
border border-[var(--color-surface)] 
focus:border-[var(--color-accent)] focus:outline-none 
text-[var(--color-text)] placeholder:text-[var(--color-text-muted)]
```

### Select / Dropdown
Same as input field. No custom select component — uses native `<select>` styled like inputs.

### Textarea
Same as input field but with `<textarea>` element.

### Labels
```
block text-sm font-medium mb-1
```
Required markers: `<span class="text-red-400">*</span>`

### Cards

#### Project Card
```
w-full text-left p-4 rounded-lg border transition-all duration-150 
hover:border-[var(--color-accent)]/50 
bg-[var(--color-panel)] hover:bg-[var(--color-panel)]/80 
border-[var(--color-surface)]
```
Selected state: `ring-2 ring-[var(--color-accent)]`

#### Asset Card
```
w-full text-left p-3 rounded-lg border transition-all duration-150 
hover:border-[var(--color-accent)]/50 
bg-[var(--color-panel)] hover:bg-[var(--color-panel)]/80 
border-[var(--color-surface)]
```
Selected state: `ring-2 ring-[var(--color-accent)]`
Thumbnail: `w-full aspect-video rounded bg-[var(--color-surface)]`

### Badges / Tags
```
inline-flex items-center px-2 py-0.5 rounded text-xs font-medium
```
Color varies by context (see Semantic Colors above). Background always uses `/20` opacity.

### Status Badge (ProjectCard)
```
inline-flex items-center px-2 py-0.5 rounded text-xs font-medium
bg-green-500/20 text-green-400  (active)
bg-yellow-500/20 text-yellow-400 (archived)
```

### Dialogs / Modals

#### Backdrop
```
fixed inset-0 bg-black/50 flex items-center justify-center z-50
```

#### Dialog Container
```
bg-[var(--color-panel)] rounded-xl shadow-2xl w-full max-w-md mx-4 
border border-[var(--color-surface)]
```

#### Dialog Header
```
flex items-center justify-between p-4 border-b border-[var(--color-surface)]
```
Title: `text-lg font-semibold`
Close button: icon button style

#### Dialog Body
```
p-4 space-y-4
```

#### Dialog Error Alert
```
p-3 rounded-lg bg-red-500/20 border border-red-500/50 text-red-400 text-sm
```

#### Dialog Footer (Actions)
```
flex justify-end gap-2 pt-2
```

### Sections (Settings page)
```
section.p-6.bg-[var(--color-panel)].rounded-xl.space-y-4
```
Section heading: `text-lg font-semibold text-[var(--color-text)]`

### Progress Bar
```
w-full h-2 bg-[var(--color-surface)] rounded-full overflow-hidden
```
Fill: `h-full bg-blue-500 transition-all duration-300` (dynamic width%)

### Icon Standards

All icons use **inline SVG** with Heroicons outline style:
```
<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="..." />
</svg>
```

| Context | Size | Class |
|---------|------|-------|
| Navigation items | 20px | `w-5 h-5` |
| Buttons (with text) | 20px | `w-5 h-5` |
| Toolbar / small | 16px | `w-4 h-4` |
| Collapsed sidebar | 24px | `w-6 h-6` |
| Empty state | 48px | `w-12 h-12` |
| Logo placeholder | 32px | `w-8 h-8` |

---

## Borders & Shapes

### Border Radius

| Value | Usage |
|-------|-------|
| `rounded` (4px) | Thumbnail corners, kind badges |
| `rounded-lg` (8px) | Buttons, inputs, cards, nav items, badges, sections, list items |
| `rounded-xl` (12px) | Dialog containers, settings sections |
| `rounded-full` | Progress bar, logo placeholder, upgrade buttons |

### Border Width

| Value | Usage |
|-------|-------|
| `border` (1px) | Cards, inputs, dialogs, panels, dividers |
| `border-2` | Not used in current design |
| `ring-2` | Selected/active state indicator (project card, asset card) |

### Border Colors

| Context | Color |
|---------|-------|
| Default | `border-[var(--color-surface)]` (#0f3460) |
| Hover on cards | `hover:border-[var(--color-accent)]/50` |
| Focus on inputs | `focus:border-[var(--color-accent)]` |
| Error | `border-red-500/50` |

---

## Shadows & Effects

### Shadows
- Dialog: `shadow-2xl`
- Other elements: no explicit shadows (rely on color contrast)

### Transitions
- Standard: `transition-colors` (color changes)
- Layout: `transition-all duration-200` (sidebar/panel collapse)
- Cards: `transition-all duration-150`

### Loading States
- Disabled buttons: `disabled:opacity-50`
- Loading text pattern: `{loading ? 'Creating...' : 'Create Project'}`
- Status bar ready: `opacity-50`

---

## Do's and Don'ts

### DO

- Use `--color-accent` (#e94560) for all primary CTAs and selected states
- Use `bg-black/50` for modal backdrops
- Use `/20` opacity for all badge/tag backgrounds
- Use `rounded-lg` for interactive elements (buttons, inputs, cards)
- Use `rounded-xl` for dialog containers
- Use Inter font family for all text
- Use inline SVG icons (Heroicons style) at standard sizes (w-4/w-5/w-6)
- Use `transition-colors` for hover state animations
- Use `ring-2 ring-[var(--color-accent)]` for selected card state
- Use `text-xs uppercase tracking-wider` for property labels
- Use semantic color badges (`/20` bg, solid text) for kind/status indicators
- Maintain `p-4` as default section padding
- Use `space-y-4` for vertical form spacing
- Use `font-mono` for IDs and code

### DON'T

- Don't use `--color-accent` for body text — it's reserved for CTAs and highlights only
- Don't use `rounded-full` on buttons (except progress bars and logos)
- Don't introduce new background colors outside the canvas/panel/surface hierarchy
- Don't use borders heavier than 1px (except ring-2 for selection)
- Don't use opacity values other than `/20`, `/50`, `/80` for color variants
- Don't use external icon libraries — maintain inline SVG consistency
- Don't use light colors or white backgrounds — this is a dark-only design
- Don't use `shadow` on non-dialog elements — rely on color layering instead
- Don't mix font families — Inter only (plus system monospace for code)
- Don't use font sizes outside the defined scale
- Don't place primary buttons on the left side of action groups — use `justify-end`
- Don't use more than 3 levels of heading hierarchy (H1, H2, H3/body)

---

## Design Tokens Reference (for Stitch)

```json
{
  "colors": [
    {"name": "canvas", "value": "#1a1a2e", "role": "Root background, input fields"},
    {"name": "panel", "value": "#16213e", "role": "Sidebar, dialogs, cards, sections"},
    {"name": "surface", "value": "#0f3460", "role": "Borders, elevated surfaces, hover states"},
    {"name": "accent", "value": "#e94560", "role": "Primary buttons, selected ring, CTAs"},
    {"name": "text", "value": "#eaeaea", "role": "Primary text"},
    {"name": "text-muted", "value": "#9ca3af", "role": "Secondary text, metadata"},
    {"name": "success", "value": "#22c55e", "role": "Active status, success"},
    {"name": "warning", "value": "#eab308", "role": "Archived status, audio kind"},
    {"name": "error", "value": "#ef4444", "role": "Errors, validation"},
    {"name": "info", "value": "#3b82f6", "role": "Image kind, info badges"},
    {"name": "purple", "value": "#a855f7", "role": "Pro tier, tileset kind"},
    {"name": "orange", "value": "#f97316", "role": "Material kind"},
    {"name": "pink", "value": "#ec4899", "role": "Voice kind"}
  ],
  "typography": {
    "fontFamily": "Inter",
    "headings": {
      "h1": {"fontSize": "24px", "fontWeight": "bold"},
      "h2": {"fontSize": "18px", "fontWeight": "600"},
      "h3": {"fontSize": "18px", "fontWeight": "600"}
    },
    "body": {"fontSize": "16px", "fontWeight": "400", "lineHeight": "1.5"},
    "small": {"fontSize": "14px"},
    "caption": {"fontSize": "12px", "fontWeight": "500", "textTransform": "uppercase", "letterSpacing": "0.05em"}
  },
  "shapes": {"borderRadius": "8px"},
  "appearance": {"colorMode": "DARK", "elevation": "color-layering"}
}
```
