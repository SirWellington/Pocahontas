import { test, expect } from "./fixtures";

/**
 * Toolbar tests.
 *
 * NOTE: The toolbar has two states:
 *   1. No catalog loaded — shows only "Open Catalog" button
 *   2. Catalog loaded — shows panel toggle buttons + Import button
 *
 * Testing state 2 requires the Zustand store to have a `path` set,
 * which in normal usage happens after the user picks a file via Tauri's
 * native file dialog. Playwright cannot trigger native dialogs.
 *
 * Workaround: modify App.tsx or useCatalog.ts to expose the store on window:
 *   `window.__CATALOG_STORE__ = useCatalogStore`
 * Then use page.evaluate() to set state in tests.
 */
test.describe("Toolbar", () => {
  test.beforeEach(async ({ app }) => {
    await app.gotoLanding();
  });

  test("shows logo and app name", async ({ page }) => {
    // The toolbar always renders the logo area
    await expect(page.getByText("Praetorian", { exact: true })).toBeVisible();
  });

  test("Open Catalog button visible when no catalog loaded", async ({ page }) => {
    // In toolbar (not the landing page button) — the toolbar has its own Open Catalog
    const toolbarBtn = page.locator(
      '[class*="h-10"] >> text="Open Catalog"'
    );
    await expect(toolbarBtn).toBeVisible();
  });

  test("panel toggle buttons NOT visible when no catalog loaded", async ({ page }) => {
    // These only appear when path exists in the store
    const hideSidebar = page.getByRole("button", { name: /Hide.*Sidebar/i });
    const hideDetails = page.getByRole("button", { name: /Hide.*Details/i });
    await expect(hideSidebar).not.toBeVisible();
    await expect(hideDetails).not.toBeVisible();
  });

  test("Import button NOT visible when no catalog loaded", async ({ page }) => {
    // The Import button only appears in the toolbar when a catalog is open
    const importBtn = page.locator('[class*="h-10"] >> text="Import"');
    await expect(importBtn).not.toBeVisible();
  });

  test.describe("With catalog loaded (requires store mocking)", () => {
    test.skip(
      true,
      "Requires Zustand store to be exposed on window. See toolbar.spec.ts comments."
    );

    test("shows Hide Sidebar and Hide Details buttons", async ({ page }) => {
      // Uncomment after adding `window.__CATALOG_STORE__ = useCatalogStore` in useCatalog.ts
      // await page.evaluate(() => {
      //   (window as any).__CATALOG_STORE__.setState({ path: '/fake/catalog.praetorian' });
      // });
      const hideSidebar = page.getByRole("button", { name: /Hide.*Sidebar/i });
      const hideDetails = page.getByRole("button", { name: /Hide.*Details/i });
      await expect(hideSidebar).toBeVisible();
      await expect(hideDetails).toBeVisible();
    });

    test("shows Import button in toolbar", async ({ page }) => {
      // Uncomment after adding window exposure
      // await page.evaluate(() => {
      //   (window as any).__CATALOG_STORE__.setState({ path: '/fake/catalog.praetorian' });
      // });
      const importBtn = page.locator('[class*="h-10"] >> text="Import"');
      await expect(importBtn).toBeVisible();
    });
  });
});
