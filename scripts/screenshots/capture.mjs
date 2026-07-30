/**
 * Captures the marketing screenshots used by the README and the landing page.
 *
 *   pnpm screenshots
 *
 * It boots the real Vite frontend with a mocked Tauri IPC layer (mock-tauri.js)
 * backed by demo-data.mjs, drives each screen with Playwright, and writes 2x
 * PNGs framed in macOS window chrome to site/public/screenshots/.
 *
 * Fidelity note: the app inherits the OS UI font, so run this on macOS to get
 * SF Pro. On other platforms install Inter and alias it to `system-ui` in
 * fontconfig — that is what the committed screenshots were captured with.
 */
import { spawn } from "node:child_process";
import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";
import { AUDIO_DURATION_SECONDS, FIXTURES, LIVE_TRANSCRIPT } from "./demo-data.mjs";

const here = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(here, "../..");
const outDir = path.join(repoRoot, "site/public/screenshots");

const PORT = Number(process.env.SCREENSHOT_PORT ?? 1420);
const BASE_URL = process.env.SCREENSHOT_BASE_URL ?? `http://localhost:${PORT}`;
const WINDOW = { width: 1440, height: 900 };
const SCALE = 2;
const THEMES = ["light", "dark"];

/** One capture per screen. `prepare` drives the UI into the state we want. */
const SHOTS = [
  {
    name: "library",
    route: "/",
    ready: "text=Design Review — Onboarding Flow",
  },
  {
    name: "meeting",
    route: "/meeting/m1",
    ready: "role=tab[name='Summaries']",
    async prepare(page) {
      await page.getByTitle("Show transcript").click();
      await page.getByRole("tab", { name: "Summaries" }).click();
    },
  },
  {
    name: "insights",
    route: "/insights",
    ready: "text=Draft two copy variants for the permissions explainer",
  },
  {
    name: "chat",
    route: "/chat",
    ready: "text=Onboarding drop-off",
  },
  {
    name: "recording",
    route: "/recording",
    ready: "text=Live Transcript",
    async prepare(page) {
      await page.evaluate((segments) => {
        window.__NOOTLE_EMIT__("transcription-status", { available: true });
        window.__NOOTLE_EMIT__("transcript-update", segments);
      }, LIVE_TRANSCRIPT);
      await page.getByRole("button", { name: "Untitled Recording" }).click();
      await page.locator("input:focus").fill("Design Review — Onboarding Flow");
      await page.getByLabel("Summary template").selectOption({ label: "Standard Summary" });
      await page.getByPlaceholder("Take notes during the meeting...").fill(
        "Permissions screen is the big leak — 31% drop\n" +
          "Try inline copy instead of the docs link, one sentence above the button\n" +
          "Two variants if traffic allows, behind the existing flag\n\n" +
          "Calendar permission: move it out of onboarding? Ask again on an empty library",
      );
      await page.getByRole("button", { name: /Live Transcript/ }).click();
      // Let the timer run so the recording reads as in-progress.
      await page.waitForTimeout(8000);
    },
  },
  {
    name: "automations",
    route: "/templates",
    ready: "text=Standard Summary",
  },
];

/** Renders a screenshot inside macOS window chrome and returns the framed PNG. */
async function frame(browser, rawPng, theme) {
  const page = await browser.newPage({
    viewport: { width: WINDOW.width + 160, height: WINDOW.height + 160 },
    deviceScaleFactor: SCALE,
  });
  const dark = theme === "dark";
  await page.setContent(`
    <style>
      html, body { margin: 0; background: transparent; }
      .stage { padding: 64px 80px 96px; width: max-content; }
      .window {
        position: relative;
        width: ${WINDOW.width}px;
        border-radius: 12px;
        overflow: hidden;
        box-shadow:
          0 48px 96px -24px rgba(0, 0, 0, ${dark ? 0.75 : 0.4}),
          0 16px 40px -16px rgba(0, 0, 0, ${dark ? 0.6 : 0.28}),
          0 0 0 1px rgba(${dark ? "255, 255, 255, 0.09" : "0, 0, 0, 0.09"});
      }
      .window img { display: block; width: ${WINDOW.width}px; height: ${WINDOW.height}px; }
      .lights { position: absolute; top: 19px; left: 20px; display: flex; gap: 8px; }
      .lights span {
        width: 12px; height: 12px; border-radius: 50%;
        box-shadow: inset 0 0 0 0.5px rgba(0, 0, 0, 0.12);
      }
    </style>
    <div class="stage">
      <div class="window">
        <img src="data:image/png;base64,${rawPng.toString("base64")}" />
        <div class="lights">
          <span style="background:#FF5F57"></span>
          <span style="background:#FEBC2E"></span>
          <span style="background:#28C840"></span>
        </div>
      </div>
    </div>
  `);
  const png = await page.locator(".stage").screenshot({ omitBackground: true });
  await page.close();
  return png;
}

async function startDevServer() {
  if (process.env.SCREENSHOT_BASE_URL) return null;
  const server = spawn(
    "node",
    [path.join(repoRoot, "node_modules/vite/bin/vite.js"), "--port", String(PORT)],
    { cwd: repoRoot, stdio: "ignore" },
  );
  for (let i = 0; i < 60; i++) {
    try {
      const res = await fetch(BASE_URL);
      if (res.ok) return server;
    } catch {
      // not up yet
    }
    await new Promise((r) => setTimeout(r, 500));
  }
  server.kill();
  throw new Error(`Vite did not start on ${BASE_URL}`);
}

async function main() {
  await mkdir(outDir, { recursive: true });
  const server = await startDevServer();
  const browser = await chromium.launch({
    // Escape hatch for environments with a preinstalled Chromium.
    executablePath: process.env.CHROMIUM_EXECUTABLE || undefined,
  });

  try {
    for (const theme of THEMES) {
      const context = await browser.newContext({
        viewport: WINDOW,
        deviceScaleFactor: SCALE,
        colorScheme: theme,
        locale: "en-US",
        timezoneId: "America/Los_Angeles",
      });
      await context.addInitScript(
        `window.__NOOTLE_FIXTURES__ = ${JSON.stringify(FIXTURES)};
         localStorage.setItem("onboarding_complete", "true");
         localStorage.setItem("theme", ${JSON.stringify(theme)});
         localStorage.setItem("llm_provider", "anthropic");
         localStorage.setItem("llm_model", "claude-opus-4-6");
         localStorage.setItem("meetingViewMode", "grid");
         localStorage.setItem("nootle-sidebar-collapsed", "false");`,
      );
      await context.addInitScript({ path: path.join(here, "mock-tauri.js") });
      // The demo audio is a one-second placeholder; report the meeting's real
      // length so the player reads like a finished recording.
      await context.addInitScript(
        `Object.defineProperty(HTMLMediaElement.prototype, "duration", {
           get: () => ${AUDIO_DURATION_SECONDS},
         });`,
      );

      for (const shot of SHOTS) {
        const page = await context.newPage();
        page.on("pageerror", (err) => console.warn(`  ! ${shot.name}: ${err.message}`));
        await page.goto(`${BASE_URL}${shot.route}`, { waitUntil: "networkidle" });
        await page.addStyleTag({ content: "*, *::after { caret-color: transparent !important; }" });
        await page.waitForSelector(shot.ready, { timeout: 15_000 });
        await shot.prepare?.(page);
        // Let entrance animations settle before capturing.
        await page.waitForTimeout(1200);

        const raw = await page.screenshot();
        const framed = await frame(browser, raw, theme);
        const file = path.join(outDir, `${shot.name}-${theme}.png`);
        await writeFile(file, framed);
        console.log(`✓ ${path.relative(repoRoot, file)}`);
        await page.close();
      }
      await context.close();
    }
  } finally {
    await browser.close();
    server?.kill();
  }
}

await main();
