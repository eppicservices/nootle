# Screenshots

Generates the app screenshots used by the root `README.md` and the landing page.
Output lands in `site/public/screenshots/` as 2x PNGs framed in macOS window
chrome — the site serves them from `/screenshots/`, and the README links to the
same files.

```bash
pnpm install
pnpm exec playwright install chromium   # once
pnpm screenshots
```

## How it works

The app is a Tauri frontend, so it runs fine in a plain browser as long as
something answers its IPC calls. `capture.mjs` starts the Vite dev server and
injects `mock-tauri.js`, which implements just enough of
`window.__TAURI_INTERNALS__` to serve `demo-data.mjs` and to fire backend events
(the live recording view is driven that way). Playwright then walks each route,
clicks it into the state we want, and screenshots it in both light and dark mode.

Because the components are the real ones, a UI change shows up in the
screenshots the next time this runs — no mockup to keep in sync.

## Editing

- **Demo content** — `demo-data.mjs`. Every value is fictional. Dates are fixed
  so re-running produces the same images.
- **Which screens get captured** — the `SHOTS` array in `capture.mjs`.
- **Window chrome** — the `frame()` function in `capture.mjs`.

## Fonts

The app inherits the OS UI font, so run this on macOS to get SF Pro. Elsewhere,
install [Inter](https://rsms.me/inter/) and alias it to `system-ui` in
fontconfig — that is what the committed screenshots were captured with.

## Environment overrides

- `CHROMIUM_EXECUTABLE` — path to an existing Chromium build.
- `SCREENSHOT_BASE_URL` — capture against an already-running server instead of
  starting Vite.
- `SCREENSHOT_PORT` — port for the Vite dev server (default `1420`).
