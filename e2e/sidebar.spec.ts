import { test, expect } from "./fixtures";

/**
 * Sidebar tests.
 *
 * The sidebar only renders when useCatalogStore.path exists AND leftPanelVisible is true.
 * Without a catalog loaded, the sidebar is not in the DOM at all.
 *
 * To test the "catalog loaded" state, expose the Zustand store:
 *   In src/hooks/useCatalog.ts, add: `;(window as any).__CATALOG_STORE__ = useCatalogStore;`
 * after the `create()` call.
 */
test.describe("Sidebar", () => {
  test.beforeEach(async ({ app }) => {
    await app.gotoLanding();
  });

  test("sidebar is not visible when no catalog loaded", async ({ page }) => {
    // The sidebar has a distinctive width class: w-56 bg-[#1e1e1e]
    const sidebar = page.locator('[class*="w-56"][class*="bg-"]');
    await expect(sidebar).not.toBeVisible();
  });

  test("sidebar search input is not present on landing page", async ({ page }) => {
    await expect(page.getByPlaceholder("Search...")).not.toBeVisible();
  });

  test.describe("With catalog loaded (requires store mocking)", () => {
    test.skip(
      true,
      "Requires Zustand store to be exposed on window. See sidebar.spec.ts comments."
    );

    test("sidebar appears with search input", async ({ page }) => {
      // await page.evaluate(() => {
      //   (window as any).__CATALOG_STORE__.setState({ path: '/fake/catalog.praetorian' });
      // });
      const sidebar = page.locator('[class*="w-56"]');
      await expect(sidebar).toBeVisible();
      await expect(page.getByPlaceholder("Search...")).toBeVisible();
    });

    test("shows All Photos, Favorites, and Rated items", async ({ page }) => {
      // await page.evaluate(() => {
      //   (window as any).__CATALOG_STORE__.setState({ path: '/fake/catalog.praetorian' });
      // });
      await expect(page.getByText("All Photos")).toBeVisible();
      await expect(page.getByText("Favorites")).toBeVisible();
      await expect(page.getByText("Rated")).toBeVisible();
    });

    test("shows Folders section header", async ({ page }) => {
      // await page.evaluate(() => {
      //   (window as any).__CATALOG_STORE__.setState({ path: '/fake/catalog.praetorian' });
      // });
      await expect(page.getByText("Folders")).toBeVisible();
    });

    test("hides sidebar when toggle button clicked", async ({ page }) => {
      // await page.evaluate(() => {
      //   (window as any).__CATALOG_STORE__.setState({ path: '/fake/catalog.praetorian' });
      // });
      // await page.getByRole("button", { name: /Hide.*Sidebar/i }).click();
      // const sidebar = page.locator('[class*="w-56"]');
      // await expect(sidebar).not.toBeVisible();
    });
  });
});
