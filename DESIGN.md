# Design System Specification: Pitch Slide Aesthetic

This document defines the visual design system for **hirn** (homepage website and desktop application), inspired by the *Product Review* slide deck (`homepage/public/assets/slides/`).

The design philosophy is **minimal, high-contrast, bold, and strictly grayscale**.

---

## 1. Core Principles

1. **Monochrome High Contrast**: Strict `#171717` (near-black) and `#F0F0F0` (slide canvas light gray) palette. No color accents (no teals/blues/purples). Contrast is achieved strictly through text weight, line borders, and surface inversion.
2. **Bold Typography**: Headings use **Plus Jakarta Sans** (weights 600, 700, 800) for a confident, editorial look. Body text uses **Inter** (weights 400, 500, 600) for clean legibility. Monospace accents use **JetBrains Mono** (weights 400, 500).
3. **Card Frame Container**: Elements, hero showcases, and panels are encapsulated in framed container boxes with rounded corners (`border-radius: 15px`), matching slide container frames (`rx=15`, fill `#F0F0F0`, stroke `#171717`).
4. **Self-Hosted & Private**: 100% bundled assets. No external font CDNs (Google Fonts, etc.) or tracking scripts.

---

## 2. Color Palette & Tokens

### Light Theme (Default Slide Canvas)
| Token | HEX / Value | Purpose |
|---|---|---|
| `--bg` / `--background` | `#F0F0F0` | Main canvas background |
| `--surface` / `--card` | `#FFFFFF` | Card surfaces, inputs, modal backgrounds |
| `--primary` / `--foreground` | `#171717` | Primary text, heavy headings, active icons |
| `--secondary` / `--muted-foreground` | `#555555` | Body copy, secondary labels, metadata |
| `--weak` / `--muted` | `#999999` | Disabled text, subtle hints, placeholder text |
| `--border` / `--input` | `#E0E0E0` | Sharp structural borders, divider lines |
| `--accent` | `#171717` | High-contrast interactive elements, CTAs |
| `--accent-hover` | `#000000` | CTA hover state |

### Dark Theme
| Token | HEX / Value | Purpose |
|---|---|---|
| `--bg` / `--background` | `#0D0D0D` | Canvas background |
| `--surface` / `--card` | `#171717` | Card surfaces |
| `--popover` | `#171717` | Popovers and floating menus |
| `--primary` / `--foreground` | `#F0F0F0` | Primary text |
| `--secondary` / `--muted-foreground` | `#A0A0A0` | Body copy, metadata |
| `--weak` / `--muted` | `#555555` | Subtle hints |
| `--border` / `--input` | `#333333` | Borders |
| `--accent` | `#F0F0F0` | High-contrast interactive CTAs |
| `--accent-hover` | `#FFFFFF` | CTA hover state |

---

## 3. Typography Stack

All fonts are self-hosted inside `public/fonts/` (homepage) and `static/fonts/` (desktop app).

```css
/* Headings & Hero Titles */
font-family: 'Plus Jakarta Sans', system-ui, -apple-system, sans-serif;
font-weight: 600, 700, 800;

/* Body & UI Controls */
font-family: 'Inter', system-ui, -apple-system, sans-serif;
font-weight: 400, 500, 600;

/* Code, Terminal & Technical Accents */
font-family: 'JetBrains Mono', monospace;
font-weight: 400, 500;
```

---

## 4. Radii & Spacing Scale

Matching the slide SVG canvas proportions (`rx="15"`):

- **`--radius`**: `0.9375rem` / `15px` (Base slide container radius)
- **`--radius-sm`**: `4px` (Tags, small badges, inline code)
- **`--radius-md`**: `8px` (Buttons, inputs, dropdown items)
- **`--radius-lg`**: `12px` (Cards, popovers, sheets)
- **`--radius-xl`**: `15px` (Main slide-style container frames)
- **`--radius-2xl`**: `24px` (Hero showcases)

---

## 5. Component Patterns (shadcn-svelte aligned)

### Cards & Slide Frames
Cards use a 1px solid border (`#E0E0E0` / `#171717` in light mode), surface background (`#FFFFFF` or `#F0F0F0`), `rounded-[15px]`, and high-contrast typography inside.

```html
<div class="rounded-[15px] border border-[#171717] bg-[#F0F0F0] p-6 text-[#171717]">
  <h3 class="font-heading text-lg font-bold">Slide Frame Title</h3>
  <p class="font-sans text-sm text-[#555555]">Descriptive body content in Inter font.</p>
</div>
```

### Buttons & CTAs
- **Primary CTA**: `#171717` background with `#F0F0F0` text (inverted in dark mode). Hover: `#000000` with subtle scale/shadow.
- **Secondary CTA**: `#FFFFFF` background with `#171717` border and text.
