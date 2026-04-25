# photo_app_rs — Claude Design reference

Distilled from the Claude Design bundle at
`https://api.anthropic.com/v1/design/h/vmQ7C_iGbeWU9baIRbQuTQ`
(extracted to `/tmp/claude-design/photo-app-rs/`).

The bundle is a handoff from Claude Design: a two-tab HTML prototype
(`project/index.html`) and a chat transcript (`chats/chat1.md`). The
prototype is mocked in HTML/CSS/JS + React UMD + Tailwind CDN — it is
reference-only; the target implementation lives in our Tauri + React app.

---

## 1 · Design philosophy

- **Dark-first, photographer-grade.** Deep warm neutrals (hue ≈ 60°), not
  pure greys. "Blacks feel like paper, not screen."
- **Calm chrome, photos as hero.** Chrome steps down a surface level when
  the viewer opens so the photo lifts.
- **Single accent.** Warm amber (default `oklch(0.78 0.14 60)`). No
  gradients, no multi-color icons, no red/green/blue star ratings.
- **Native-feeling on macOS first**, but legible on Linux/Windows. Traffic
  light awareness, vibrancy-like surfaces, SF-adjacent type.
- **Keyboard-driven.** Every chrome affordance has a keyboard path.
- **Non-destructive.** "source untouched" is a brand beat shown in the UI.

Tags the design carries forward: `dark-first`, `photographer-grade`,
`native-feeling`, `keyboard-driven`, `non-destructive`.

---

## 2 · Color system (OKLCH)

Declared as CSS variables on `:root`:

```css
/* surfaces (warm-leaning neutrals, chroma ~0.006) */
--ink-0:  oklch(0.12 0.005 60);  /* deepest surface / viewer */
--ink-1:  oklch(0.16 0.006 60);  /* app canvas */
--ink-2:  oklch(0.19 0.006 60);  /* chrome (sidebar/searchbar/titlebar/status bar) */
--ink-3:  oklch(0.23 0.007 60);  /* raised / hover */
--ink-4:  oklch(0.28 0.008 60);  /* border stark */
--ink-5:  oklch(0.34 0.009 60);  /* border bright */
--hair:   oklch(1 0 0 / 0.06);   /* hairline over dark */
--hair-2: oklch(1 0 0 / 0.10);

/* foreground */
--fg-0:   oklch(0.97 0.006 80);  /* pure foreground */
--fg-1:   oklch(0.86 0.006 80);  /* primary */
--fg-2:   oklch(0.68 0.006 80);  /* secondary */
--fg-3:   oklch(0.52 0.006 80);  /* tertiary / meta */
--fg-4:   oklch(0.40 0.006 80);  /* quiet */

/* single accent */
--accent-h:    60;
--accent:      oklch(0.78 0.14 var(--accent-h));
--accent-2:    oklch(0.70 0.14 var(--accent-h));
--accent-soft: oklch(0.78 0.14 var(--accent-h) / 0.14);

/* semantic (status bar / toasts only — never chrome) */
--ok:   oklch(0.78 0.14 150);
--warn: oklch(0.78 0.14 70);
--err:  oklch(0.70 0.17 25);
```

**Viewer shift rule:** when MediaViewer opens, shell steps `ink-1 → ink-0`
and hue shifts 2–3° cooler. Tweaks expose a "viewer depth" choice:
`black | dim | jet`.

Runtime-tweakable `--accent-h` (0–360) for personalization. Default 60
(warm amber).

---

## 3 · Typography

- **UI:** Inter Tight (300/400/500/600/700), font-features `ss01`,`cv11`,
  letter-spacing tightened by class (`tight` = -0.014em, `tighter` = -0.022em).
- **Mono:** JetBrains Mono (400/500), features `zero`,`ss01`,
  `tabular-nums` — used for EXIF, filenames, sizes, counts, timestamps.
- **No third font.**

Scale:

| Role     | Size / LH / tracking / weight     |
|----------|-----------------------------------|
| Display  | 32 / 36 / -0.022em / 600          |
| H1       | 22 / 28 / -0.022em / 600          |
| H2       | 17 / 22 / -0.014em / 600          |
| Body     | 13.5 / 20 / -0.003em / 400        |
| UI       | 12.5 / 18 / -0.005em / 500        |
| Kicker   | 10.5 / 14 / 0.14em UPPER / 500    |
| Mono     | 11 / 16 / 0 / 400 (tabular-nums)  |

`wide-kicker` class: uppercase, tracked 0.14em, weight 500, 10.5px, `fg-3`
— used as section headers ("Drives", "Cameras", "Collections", "Capture",
"Exposure", "File", "Histogram").

---

## 4 · Spacing, radius, iconography

- Spacing scale: `2 · 4 · 8 · 12 · 16 · 20 · 32`.
- Radius: `4 · 6 · 10`. Window corners 10, tiles 4, controls 6.
- Icons: **Lucide, 1.5px stroke.** 14–16px in chrome, 12–13px inline.
  Icons inherit `fg-3`; brighten to `fg-0` on hover. No filled icons in
  sidebar — fills reserved for `play` and RAW badges.

---

## 5 · Motion

- Tile hover: 160ms `cubic-bezier(.2,.7,.2,1)`, `translateY(-1px)` +
  1px hairline ring. **No `opacity` dimming on hover** (it dims the photo).
- Viewer open: 200ms opacity + 6px rise. No scale-from-tile zoom.
- Sidebar collapse: 180ms ease-out, labels fade, icons stay.
- Date header: sticky with `backdrop-filter: blur(10px) saturate(130%)`.
- Command palette: 140ms drop-in from top. Respects `prefers-reduced-motion`.
- `.rise` keyframe: `opacity 0 → 1`, `translateY(4px) → 0`, 240ms.

---

## 6 · Layout

App is a bordered "window" with 10px radius. Grid top to bottom:

```
titlebar (28px) — traffic lights + centered drive name
 ├─ sidebar (240px collapsible to 56px, vibrancy surface)
 │    • app mark + version
 │    • nav: Library / Photos / Videos / Starred / Recent Imports
 │    • kicker — Drives (name, online dot)
 │    • kicker — Cameras (name, count)
 │    • kicker — Collections
 │    • footer: library meter (thin amber bar), GB, items, "indexed"
 │    • footer: "Scan a drive  ⌘O"
 │
 └─ main
       ├─ search bar (44px, vibrancy)
       │    • centered input, max 640, ⌘K chip inside
       │    • chips: All cameras · All types · Year · Sort
       │    • icon toggles: grid · map · sliders
       │
       ├─ timeline (flex-1, ink-1)
       │    • rise-animated sections per day
       │    • sticky date header (44px, blur, day title + counts + dominant camera)
       │    • if videos: dedicated 16:9 row with VIDEO kicker
       │    • then photos grid respecting real aspect ratio
       │    • tileMin: compact 108 · default 140 · roomy 180, gap 2/4/8
       │
       ├─ scrubber (36px right rail)
       │    • mono month/year labels, click to jump
       │
       └─ status bar (24px, ink-2, mono 10.5)
            online dot · items indexed · GB · thumb cache % · sidecar sync · "source · read-only"
```

Overlays: **MediaViewer** (z-30) and **Command palette** (z-40) mount on
top of the shell.

---

## 7 · Components

### Sidebar
- `<aside class="vibrancy">`, width 240 (collapsed 56). Header reserves
  76px left padding for macOS traffic lights.
- Nav row: 28px tall, 5px radius, icon + label (left) + mono count (right,
  tabular). Active state: `fg-0` text + `ink-3` background + accent icon.
- Three grouped sections under the nav — each with an uppercase kicker
  header: **Drives** (with online status dot), **Cameras** (with count),
  **Collections** (with `+` to add).
- Footer: thin amber progress meter (library fill %), total GB, item
  count + "indexed". Then the scan button (neutral raised + `⌘O` kbd),
  **not blue**.

### SearchBar
- Centered search input in a rounded pill at `ink-3` background with
  hairline. Focus ring uses the amber accent.
- Inline `⌘K` chip opens the command palette.
- Chip buttons replace native `<select>`s: `All cameras`, `All types`,
  year, `Newest`. Icon-only buttons for grid / map / sliders on the far
  right.
- Debounce: **120ms** (not 300ms — local FTS5 is fast).

### Timeline
- Per day: sticky date header (blur), VIDEO row if any, then photos
  grid. Photos and videos **never interleave**.
- Real EXIF aspect ratio: `1/1`, `3/2`, or `2/3`. Mixed days get natural
  photographic rhythm.
- Video tiles: 8% letterbox bars (top + bottom) + mono duration pill
  `mm:ss` bottom-right + play glyph. **No "Video" text pill.**
- Tile extras: `RAW` badge bottom-left for ARW, star dots top-left
  for rated items, `×N` chip top-right for bursts.
- Selection: 2px amber ring + soft drop shadow. Focus-visible: same ring.

### Scrubber
- 36px right rail, bordered-left. Mono month + year labels, `MAR '26`.
  On click: `virtuoso.scrollToIndex(groupIdx)`.
- Design calls out a "wow" behavior: while dragging, grid blurs to 8px
  and month label overlays large. Snap on release.

### MediaViewer (overlay, z-30)
- Top chrome 44px with backdrop blur, auto-hide after 2s idle. Close
  (left) · filename+format+size mono-centered · star + info (right).
- Prev / next arrows as floating circular buttons (left/right edges).
  Bind `ArrowLeft`, `ArrowRight`, `Escape`, `I`, `J`, `K`.
- Photo gets `box-shadow: 0 30px 80px -20px oklch(0 0 0 / 0.7), 0 0 0 1px
  oklch(1 0 0 / 0.05)` to lift off `ink-0`.
- **Do NOT close on backdrop click** (touchpad misfires). Close only on
  explicit button or `Escape`.
- Bottom filmstrip 64px: ~24 neighbors, current ringed in accent.
- Right info panel (320px wide, opens on `I`). Sections in this order:
  **Capture** (camera + lens) · **Exposure** (ISO / ƒ / shutter / focal,
  big mono tabular) · **File** (filename, format, size, drive, path) ·
  **Histogram** (48 bars from thumb RGBA). Footer: "non-destructive ·
  source untouched" + green dot.

### Command palette (⌘K, overlay z-40)
- 560px card, 10px radius, drop-in from top at 12vh, backdrop blur.
- Command model: `{ category, name, run(appStore), kb: [...] }`.
- Starter commands: Go to Today · Jump to year… · Only RAW (.ARW) ·
  Only starred · Scan a drive (`⌘O`) · Reveal in Finder (`⌘⇧R`) ·
  Toggle info panel (`I`) · Toggle sidebar (`⌘\`).

### Status bar
- 24px, `ink-2`, mono `10.5px`, `fg-4`. Reads a Zustand slice subscribed
  to `scan_progress` + `device_connected` events.
- Fields: online dot · items indexed · library GB · thumbnails cached % ·
  sidecar db sync · "source · read-only".

### Tile (shared)
- `.tile` class: `overflow: hidden`, radius 4, `ink-3` background, inset
  hairline via `::after`, 160ms transform. Selected = 2px accent ring.
- Placeholder: striped `thumb-ph` with seed-based hue for demo only — in
  real app replace with `<img src="data:image/jpeg;base64,{thumb_small_b64}">`.

---

## 8 · Wow layer (signature touches)

1. **Dominant-color frame tint.** On viewer open, bleed a 24px wash of
   the image's dominant color into top + bottom chrome at 6% opacity,
   800ms ease. Requires a new `dominant_color TEXT` column on
   `media_files` (compute at thumbnail time, store hex).
2. **Scrubber that actually scrubs.** Drag the right rail → grid blurs
   to 8px, huge mono month labels overlay, snap on release.
3. **Command palette at ⌘K** (as above).
4. **Portable sidecar banner.** First time the user points at a drive,
   a small inline card offers "create a portable sidecar here" — one
   line explanation that the catalog lives with the files, not in a
   cloud.

---

## 9 · What to NOT do

- No skeuomorphism (polaroids, sprocket-hole film strips, leather, paper).
- No colorful chrome. One accent, period.
- No Material elevation (drop-shadowed cards everywhere). Depth =
  hairlines + vibrancy.
- No "Video" text pill.
- No progress bars you don't need. Scan status is a status-bar line.
- No onboarding modal. The pitch is "point at a drive, browse".
- No gradient on the accent.
- No native `<select>` dropdowns — they break the aesthetic on macOS.
- No closing the viewer on backdrop click.

---

## 10 · Bugs / fixes the design calls out in our current code

- **`Timeline.tsx` misuses `GroupedVirtuoso`.** `itemContent` returns a
  single-child `<MediaGrid>` per virtuoso item, which defeats CSS grid.
  Fix: either switch to `VirtuosoGrid` per date group, or keep
  `GroupedVirtuoso` but chunk `group.media` into rows of N tiles where
  `N = floor(containerWidth / tileMin)`.
- **`MediaGrid.tsx` uses `aspect-square`** and a black pill. Replace
  with real `width/height` aspect ratio + letterboxed video tiles.
- **`MediaGrid.tsx` uses `hover:opacity-80`** — dims the photo. Swap for
  `translateY(-1px)` + accent ring.
- **`Sidebar.tsx` uses `bg-blue-600`** scan button. Should be neutral
  raised + `⌘O`.
- **`SearchBar.tsx` uses native `<select>`**. Replace with chip +
  popover. Also drop debounce to 120ms.
- **`MediaViewer.tsx` closes on any `onClick`**. Remove that handler.
- **Only `Esc` and `I` bound.** Add `ArrowLeft`/`Right`/`J`/`K`.

---

## 11 · Tweaks (runtime direction variants)

The prototype exposes these, defaulting to: `density: "roomy"`,
`videoTreatment: "pill"`, `viewerDepth: "black"`, `accentHue: 142`.

- **Density:** `compact` (108px tiles, 2px gap) / `default` (140px, 4px) /
  `roomy` (180px, 8px).
- **Video treatment:** `letterbox` (8% bars) / `play` (glyph only) /
  `pill` (old behavior, for comparison).
- **Viewer depth:** `black` (pure #000) / `dim` (`ink-0`) / `jet` (deeper
  `oklch(0.06 0.003 60)`).
- **Accent hue:** slider over full color wheel.

These are for exploration — they don't all need to ship, but the token
seams should exist in CSS vars so a user preference could toggle them.

---

## 12 · Implementation stack implications

- Keep TailwindCSS, but define the full token layer in CSS variables
  (above) at `:root` so OKLCH values survive Tailwind's compiler. Use
  Tailwind utilities for layout; use `style={{}}` or small helper
  classes (`.fg-1`, `.surface-2`, `.vibrancy`, `.hair`) for color tokens.
- Add `lucide-react`.
- Add Inter Tight + JetBrains Mono via Google Fonts or self-hosted.
- `react-virtuoso` stays — switch from `GroupedVirtuoso` (1-child rows)
  to either `VirtuosoGrid` per group or `GroupedVirtuoso` with row
  chunking.
- Zustand additions implied by the design:
  - `selectedMedia: number[]` (multi-select on tiles)
  - `sideCollapsed: boolean`, `viewerInfoOpen: boolean`
  - `cmdOpen: boolean`
  - `density`, `videoTreatment`, `viewerDepth`, `accentHue` (persisted)
- DB: new column `dominant_color TEXT` on `media_files` (migration 003)
  if we ship the color-tint wow feature. Add it when the thumbnail
  pipeline is touched anyway.

---

## 13 · Source

- README (bundle): `/tmp/claude-design/photo-app-rs/README.md`
- Full prototype + doc: `/tmp/claude-design/photo-app-rs/project/index.html`
- Chat transcript: `/tmp/claude-design/photo-app-rs/chats/chat1.md`
- Re-fetchable via `WebFetch`:
  `https://api.anthropic.com/v1/design/h/vmQ7C_iGbeWU9baIRbQuTQ`
  (returns a gzipped tarball; binary saved under tool-results).
