# Huuk by teenueng — production frontend

A production-grade build of the Huuk by teenueng frontend. It grew out of the original
wireframe prototype (now archived at `../../archive/wireframe/`) and is the canonical app —
`dev.sh`, Docker Compose and CI all run this folder.

| | Prototype (`../../archive/wireframe/`) | Production (`this folder`) |
|---|---|---|
| Purpose | Low-fidelity, structure-first wireframe (reference only) | Client-ready product UI (canonical) |
| Visual language | Black & white, hard borders, no shadows | Token-driven theme: indigo accent, slate neutrals, soft borders, shadows, radius scale |
| Functionality | 10 deck screens, Guide overlay, popups, Content Studio, public reader, auth, permissions, locks, platforms | **Identical** — same components, queries, API contract and tests |
| Theme readiness | Token block in `style.css` | Semantic tokens ready for the client CI theming design (`../../business/features/theming/theming.md`) |
| Package | `wireframe` | `content-planner-production` |

The app talks to the Rust backend through a data-source switch. `src/api/index.ts` picks it
the same way: unset `VITE_API_URL` → in-memory mock; set → the Rust API (`../server`).

## Quickstart

```sh
bun install          # or: npm install
bun run dev          # Vite dev server
```

Point it at the Rust backend:

```sh
# production/.env.local
VITE_API_URL=http://localhost:8787
```

```sh
bun run build        # typecheck (vue-tsc) + production build
bun run test         # unit tests (Vitest + happy-dom)
bun run typecheck    # types only
```

Demo accounts (demo mode): `owner@studio.local` / `editor@studio.local` / `client@studio.local`,
password `demo1234`.

## Structure

```text
src/
  app/                  entry: App.vue, main.ts, style.css, router/
  components/
    ui/                 presentational primitives (Chip, CardBase, KpiCard, …)
    layout/             AppShell, ScreensDeck
    overlays/           popups + auth (LoginModal, PlatformLogin, SettingsPanel, …)
    tables/             MasterTable, TxnTable
    editor/             ContentEditor
    live/               LiveSection
  core/                 app logic: auth, session, queries, i18n, theme, screens, …
  api/                  backend contract + HTTP client (pages never touch mock/)
  mock/                 in-memory backend used when VITE_API_URL is unset
  pages/                one file per route/screen
  tests/                Vitest suites (one per feature area)
```

Import with the `@` alias (`@/*` → `src/*`), e.g. `@/components/ui`,
`@/core/queries`, `@/mock/db` — never with long `../../` chains.

## Design system

All visual values are CSS custom properties at the top of `src/app/style.css`. Components consume
tokens only — no raw hex values outside that block (the provider login pages keep their own chrome
on purpose, so the simulated Facebook/Google/TikTok pages still look like the real ones).

| Token | Value | Use |
|---|---|---|
| `--surface` / `--surface-2` | `#ffffff` / `#f8fafc` | Page and card surfaces / quiet fills |
| `--ink` / `--muted` / `--faint` | `#0f172a` / `#64748b` / `#e2e8f0` | Text / secondary text / borders |
| `--wash` / `--line` | `#f1f5f9` / `#e2e8f0` | Hover fills / rules |
| `--accent` / `--accent-hover` / `--accent-wash` | `#4f46e5` / `#4338ca` / `#eef2ff` | Actions, active nav, focus, selection |
| `--success` / `--warning` / `--danger` | `#059669` / `#d97706` / `#dc2626` | Status dots, conflict banner, errors |
| `--radius` / `--radius-sm` / `--radius-lg` | `12px` / `8px` / `16px` | Cards / controls / popups |
| `--shadow-sm` … `--shadow-lg` | slate-tinted | Elevation scale |

Typography: `"Noto Sans Thai", "Inter", "Segoe UI", system-ui` — body 15px, h1 26px/700, labels
13px/650, table text 14px with tabular numerals.

Preserved from the wireframe (do not change casually — they are product behavior, not styling):
the **1:3:3:1 deck viewport**, snap geometry, the full-page `width`/`left` animation, container
queries at 720/1100 px, and the responsive breakpoints at 860/560/640 px. See
[`../docs/ARCHITECTURE.md`](../docs/ARCHITECTURE.md).

## Non-negotiable rules

1. **One sheet, one screen.** Screens are registered in `src/core/screens.ts` — nowhere else.
2. **Only `PlannerPage` writes posts.** Every other view is a read projection.
3. **Computed fields are guarded.** Editing one opens `ProtectedModal` (Cancel default).
4. **Navigation is a slide deck**, not page swaps. The URL follows the slide.
5. **All color comes from tokens.** Add tokens in `src/app/style.css`; never hard-code hex in components.
6. **Provider surfaces are never themed.** `PlatformLogin` keeps the platform's own chrome.
7. **The backend swap point is `src/api/index.ts`.** Pages never import `src/mock/` directly.
8. **Import by group, not by relative path.** `import { Chip } from '@/components/ui'`,
   `import { usePosts } from '@/core/queries'` — the `@` alias points at `src/`.

## Documentation

| Doc | Read it when… |
|---|---|
| [`../docs/ARCHITECTURE.md`](../docs/ARCHITECTURE.md) | You need to understand data flow, the deck, animation or layout |
| [`../docs/COMPONENTS.md`](../docs/COMPONENTS.md) | You need to find what a component/page does |
| [`../docs/GUIDES.md`](../docs/GUIDES.md) | You are making a change (how-tos + gotchas) |
| [`../docs/CHANGELOG.md`](../docs/CHANGELOG.md) | You want the build history and decisions |
| [`../server/README.md`](../server/README.md) | You need the API reference and production path |

Deployment: [`../../docs/DEPLOYMENT.md`](../../docs/DEPLOYMENT.md). `Dockerfile` (built from the repo-root
context: `docker build -f app/Dockerfile .`) plus `ops/nginx/` build this folder as a static image; the
backend can also serve it directly with `STATIC_DIR=app/dist`.

## Desktop (Tauri, no browser needed)

`../desktop/` wraps this frontend plus the Rust backend (`../server`, bundled as a
sidecar) into a standalone Huuk desktop app. The sidecar runs localhost-only on
port 8787 with the snapshot at the OS app-data dir and single-user demo auth,
so the app works fully offline; data persists across restarts.

```sh
npm run desktop:sidecar  # one-time (and after any server change): stage the sidecar binary
npm run desktop:dev      # Vite + Tauri window with the local sidecar
npm run desktop:build    # sidecar + frontend bundle → installer (.deb/.AppImage/…)
```

On this machine (CachyOS, no sudo) the system web libs live in
`~/.local/share/tauri-sys` instead, so the shell links via
`desktop/.cargo/config.toml` and runs via `desktop/huuk.sh` (see also the Huuk
entry in the app launcher). Rebuild the binary the same no-sudo way with:

```sh
npm run desktop:build:local
```

Why the `:local` variant: the Tauri CLI injects the `custom-protocol` cargo
feature on `tauri build` (without it the window falls back to the Vite
dev-server URL); direct `cargo build` needs the feature passed explicitly.
`~/.local/share/tauri-sys/lib/webkit-path-shim.so` additionally remaps WebKit's
hardcoded helper paths to the staged copy — a single future
`sudo pacman -S webkit2gtk-4.1 xdg-dbus-proxy` removes that shim.

Linux needs the Tauri system libraries once (no sudo here, so run this yourself):

```sh
sudo apt update && sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file \
  libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

`npm run build:desktop` bakes `VITE_API_URL=http://127.0.0.1:8787` into `dist/`
(the regular web `npm run build` is untouched). Icons live in
`../desktop/icons/` (regenerate: `npx tauri icon ../desktop/icon.svg`).
