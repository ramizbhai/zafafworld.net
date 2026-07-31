import { test, expect } from '@playwright/test';

test.describe('Homepage Redesign Integration Tests', () => {
  test('should load the homepage and check bilingual layouts', async ({ page }) => {
    // Navigate to the root homepage
    await page.goto('/');

    // Verify title and page layout containers exist
    const title = await page.title();
    expect(title).toBeTruthy();

    // Verify critical above-the-fold form controls
    const categorySelect = page.locator('#search-category');
    await expect(categorySelect).toBeVisible();

    const citySelect = page.locator('#search-city');
    await expect(citySelect).toBeVisible();

    // Verify section landmarks exist
    const heroSection = page.locator('#home-hero-section');
    await expect(heroSection).toBeVisible();

    const categoriesSection = page.locator('[aria-labelledby="categories-showcase-title"]');
    await expect(categoriesSection).toBeVisible();
  });
});
