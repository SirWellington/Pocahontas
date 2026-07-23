import { test as base } from "@playwright/test";

/**
 * Base fixture for Praetorian E2E tests.
 *
 * The app runs on the Vite dev server (localhost:1420).
 * For local Playwright testing, run `npm run dev` first.
 *
 * IMPORTANT LIMITATIONS:
 * - Tauri native APIs (file dialogs, fs access, shell commands) do NOT work
 *   in Playwright's browser context. They only work inside the actual Tauri runtime.
 * - Zustand store state cannot be manipulated from outside unless explicitly
 *   exposed on `window`. Tests that require a loaded catalog should either:
 *   1. Expose the store on window for testing: `window.__CATALOG_STORE__ = useCatalogStore`
 *   2. Run integration tests through `tauri dev` with WebDriver/tauri-driver instead
 */
export interface AppFixture {
  app: {
    /** Navigate to the app root. Use before each test that needs a fresh page. */
    gotoLanding: () => Promise<void>;
  };
}

export const test = base.extend<AppFixture>({
  app: async ({ page }, use) => {
    await use({
      gotoLanding: async () => {
        await page.goto("/");
      },
    });
  },
});

export { expect } from "@playwright/test";
