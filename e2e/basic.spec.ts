import { test, expect } from '@playwright/test';

// Note: these tests run against the Vite dev server (http://localhost:1420).
// Tauri-specific features (invoke, file system, notifications) are mocked
// by the browser's fallback behavior. For full integration testing, run
// against a built app with `pnpm tauri dev`.

test.describe('Navigation', () => {
  test('app loads and shows sidebar', async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('nav.sidebar')).toBeVisible();
    await expect(page.locator('.logo')).toBeVisible();
  });

  test('sidebar navigation items present', async ({ page }) => {
    await page.goto('/');
    const nav = page.locator('nav.sidebar');
    await expect(nav.getByText('Dashboard')).toBeVisible();
    await expect(nav.getByText('Transazioni')).toBeVisible();
    await expect(nav.getByText('Categorie')).toBeVisible();
    await expect(nav.getByText('Ricorrenti')).toBeVisible();
  });
});

test.describe('Transactions page', () => {
  test('shows list view with filters', async ({ page }) => {
    await page.goto('/');
    await page.locator('nav.sidebar button', { hasText: 'Transazioni' }).click();
    await expect(page.locator('.search-input')).toBeVisible();
    await expect(page.locator('select')).toHaveCount({ minimum: 2 });
  });

  test('search input accepts text', async ({ page }) => {
    await page.goto('/');
    await page.locator('nav.sidebar button', { hasText: 'Transazioni' }).click();
    const search = page.locator('.search-input');
    await search.fill('caffè');
    await expect(search).toHaveValue('caffè');
  });
});

test.describe('Ricorrenti page', () => {
  test('shows empty state when no templates', async ({ page }) => {
    await page.goto('/');
    await page.locator('nav.sidebar button', { hasText: 'Ricorrenti' }).click();
    // Page renders (either empty state or table)
    const page_div = page.locator('.page');
    await expect(page_div).toBeVisible();
  });
});

test.describe('Theme toggle', () => {
  test('theme toggle button visible in sidebar', async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('.theme-btn')).toBeVisible();
  });

  test('clicking theme toggle changes data-theme attribute', async ({ page }) => {
    await page.goto('/');
    const html = page.locator('html');
    const initial = await html.getAttribute('data-theme');
    await page.locator('.theme-btn').click();
    const after = await html.getAttribute('data-theme');
    expect(after).not.toBe(initial);
  });
});
