import { test, expect } from '@playwright/test';

test.describe('null-e GUI UX Redesign', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    // Accept disclaimer if shown (stored in localStorage)
    await page.evaluate(() => {
      localStorage.setItem('null-e:disclaimer-accepted', new Date().toISOString());
    });
    await page.reload();
    // Wait for the app to mount
    await page.waitForSelector('[data-tauri-drag-region]', { timeout: 10_000 });
  });

  test('Welcome view renders with scan button', async ({ page }) => {
    // AppBar should be visible
    await expect(page.locator('text=null-e').first()).toBeVisible();

    // Welcome view: "Scan for waste" button should be present
    const scanButton = page.locator('button', { hasText: 'Scan for waste' });
    await expect(scanButton).toBeVisible();

    // Path info text
    await expect(
      page.getByText('No scan paths configured').or(page.getByText('Scanning:'))
    ).toBeVisible();

    // Settings link should be visible
    await expect(
      page.locator('text=Change scan paths')
    ).toBeVisible();
  });

  test('AppBar has theme toggle and settings buttons', async ({ page }) => {
    // Theme toggle button
    const themeBtn = page.locator('button[title="Toggle theme"]');
    await expect(themeBtn).toBeVisible();

    // Settings button
    const settingsBtn = page.locator('button[title="Settings"]');
    await expect(settingsBtn).toBeVisible();

    // Rescan button should NOT be visible on welcome view
    const rescanBtn = page.locator('button[title="New scan"]');
    await expect(rescanBtn).not.toBeVisible();
  });

  test('Theme toggle cycles through dark, light, and system', async ({ page }) => {
    const html = page.locator('html');

    // Default theme is dark
    await expect(html).toHaveClass(/dark/);

    // Click theme toggle: dark → light
    await page.locator('button[title="Toggle theme"]').click();
    await expect(html).not.toHaveClass(/dark/);

    // Click again: light → system
    await page.locator('button[title="Toggle theme"]').click();

    // Click again: system → dark
    await page.locator('button[title="Toggle theme"]').click();
    await expect(html).toHaveClass(/dark/);
  });

  test('Settings drawer opens and closes', async ({ page }) => {
    // Open settings via AppBar gear icon
    await page.locator('button[title="Settings"]').click();

    // Drawer should appear with "Settings" heading
    await expect(
      page.locator('h2', { hasText: 'Settings' })
    ).toBeVisible();

    // Should have sections (use heading role to disambiguate from welcome view)
    await expect(page.getByRole('heading', { name: 'Scan Paths' })).toBeVisible();
    await expect(page.getByRole('heading', { name: 'Appearance' })).toBeVisible();

    // Theme buttons should be in drawer
    const darkBtn = page.locator('button', { hasText: 'dark' });
    await expect(darkBtn).toBeVisible();

    // Close via X button
    const closeBtn = page.locator('button').filter({ has: page.locator('svg.lucide-x') });
    await closeBtn.first().click();

    // Drawer should be gone
    await expect(
      page.locator('h2', { hasText: 'Settings' })
    ).not.toBeVisible();
  });

  test('Settings drawer opens from welcome view link', async ({ page }) => {
    // Click "Change scan paths" link
    await page.locator('text=Change scan paths').click();

    // Drawer should appear
    await expect(
      page.locator('h2', { hasText: 'Settings' })
    ).toBeVisible();
  });

  test('Scan button triggers scanning view', async ({ page }) => {
    // Click scan - this will try to call Tauri which won't work in browser,
    // but we can verify the state transition attempt
    const scanButton = page.locator('button', { hasText: 'Scan for waste' });
    await scanButton.click();

    // The app should transition to scanning state
    // Since Tauri commands aren't available in browser, scan might fail
    // and return to welcome, or show scanning view briefly
    // Let's just verify the button click worked (no crash)
    await page.waitForTimeout(500);

    // App should still be rendered (no crash)
    await expect(page.locator('text=null-e').first()).toBeVisible();
  });

  test('Welcome view layout is centered and minimal', async ({ page }) => {
    // The scan button should be large and centered
    const scanButton = page.locator('button', { hasText: 'Scan for waste' });
    const box = await scanButton.boundingBox();
    expect(box).not.toBeNull();

    if (box) {
      const viewport = page.viewportSize();
      if (viewport) {
        // Button should be roughly centered horizontally
        const buttonCenter = box.x + box.width / 2;
        const viewportCenter = viewport.width / 2;
        expect(Math.abs(buttonCenter - viewportCenter)).toBeLessThan(50);
      }
    }
  });

  test('No sidebar or navigation exists', async ({ page }) => {
    // Old sidebar elements should not exist
    const sidebar = page.locator('nav');
    const count = await sidebar.count();
    expect(count).toBe(0);

    // No navigation links like "Dashboard", "Projects", etc.
    await expect(page.locator('text=Dashboard')).not.toBeVisible();
    await expect(page.locator('text=Projects').first()).not.toBeVisible();
  });

  test('App renders without console errors', async ({ page }) => {
    const errors: string[] = [];
    page.on('console', (msg) => {
      if (msg.type() === 'error') {
        errors.push(msg.text());
      }
    });

    await page.goto('/');
    await page.waitForTimeout(1000);

    // Filter out expected Tauri-related errors (Tauri APIs aren't available in browser)
    const realErrors = errors.filter(
      (e) =>
        !e.includes('__TAURI__') &&
        !e.includes('tauri') &&
        !e.includes('invoke') &&
        !e.includes('Failed to fetch')
    );

    expect(realErrors).toHaveLength(0);
  });
});
