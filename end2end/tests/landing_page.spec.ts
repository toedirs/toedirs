import { test, expect } from "@playwright/test";

test("homepage has title and links to intro page", async ({ page }) => {
  await page.goto("http://localhost:8479/");

  await expect(page).toHaveTitle("Welcome to Toedi");

  await expect(page.locator("div[class='navbar-brand'] > a")).toHaveText("Toedi");
});
