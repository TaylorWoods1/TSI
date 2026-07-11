import { test, expect } from "@playwright/test";

test.describe("Spyder configurator", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await expect(page.getByText("Spyder")).toBeVisible();
    await expect(page.getByText(/Classify:/)).toBeVisible({ timeout: 20_000 });
  });

  test("loads venue with four anchors", async ({ page }) => {
    await expect(page.getByText("Anchor 1", { exact: true })).toBeVisible();
    await expect(page.getByText("Anchor 4", { exact: true })).toBeVisible();
  });

  test("pulley cable model shows radius field", async ({ page }) => {
    await page.locator("label", { hasText: "Cable model" }).locator("..").locator("select").selectOption("pulley");
    await expect(page.getByText("Pulley radius (m)")).toBeVisible({ timeout: 5000 });
  });

  test("simulate plan line and play", async ({ page }) => {
    await page.getByRole("button", { name: "Simulate" }).click();
    await page.locator("summary", { hasText: "Line plan" }).click();
    await page.getByRole("button", { name: "Plan line" }).click();
    await page.getByRole("button", { name: "Play" }).click();
    await expect(page.getByRole("button", { name: "Pause" })).toBeVisible({
      timeout: 8000,
    });
  });

  test("run mock connect and estop", async ({ page }) => {
    await page.getByRole("button", { name: "Run" }).click();
    await page.getByRole("button", { name: "Connect" }).click();
    await expect(page.getByRole("button", { name: "Disconnect" })).toBeVisible({
      timeout: 10_000,
    });
    await page.getByRole("button", { name: "E-STOP" }).click();
    await expect(page.getByText("E-STOP ACTIVE")).toBeVisible();
  });

  test("motor mapping panel saves per-cable axes", async ({ page }) => {
    await page.locator("summary", { hasText: "Motor mapping (per cable)" }).click();
    await page.getByRole("button", { name: "Save motor mapping" }).click();
    await expect(page.getByText("Cable 1")).toBeVisible();
  });

  test("multiboard mock connect", async ({ page }) => {
    await page.getByRole("button", { name: "Run" }).click();
    await page.locator("label", { hasText: "Backend" }).locator("..").locator("select").selectOption("multiboard");
    await expect(page.getByText("Mock hardware (dry-run)")).toBeVisible();
    await page.getByRole("button", { name: "Connect" }).click();
    await expect(page.getByRole("button", { name: "Disconnect" })).toBeVisible({
      timeout: 10_000,
    });
  });

  test("field calibration capture and export venue TOML", async ({ page }) => {
    await page.locator("summary", { hasText: "Field calibration" }).click();
    await page.getByRole("button", { name: "Capture calibration" }).click();

    const calRes = await page.request.get("/calibration");
    expect(calRes.ok()).toBeTruthy();
    const cal = (await calRes.json()) as { home_lengths_m: number[] };
    expect(cal.home_lengths_m).toHaveLength(4);

    const tomlRes = await page.request.get("/calibration/venue_toml");
    expect(tomlRes.ok()).toBeTruthy();
    const { toml } = (await tomlRes.json()) as { toml: string };
    expect(toml).toContain("[[anchors]]");
    expect(toml).toContain("[home]");

    await page.getByRole("button", { name: "Export venue TOML" }).click();
  });

  test("venue TOML round-trip preserves anchors and classify", async ({ page }) => {
    const venueBefore = await page.request.get("/venue");
    expect(venueBefore.ok()).toBeTruthy();
    const { classify: classifyBefore, venue: venueBeforeData } = (await venueBefore.json()) as {
      classify: string;
      venue: { anchors: unknown[] };
    };
    expect(venueBeforeData.anchors).toHaveLength(4);

    const tomlRes = await page.request.get("/venue/toml");
    expect(tomlRes.ok()).toBeTruthy();
    const { toml } = (await tomlRes.json()) as { toml: string };

    await page.locator("label", { hasText: "Preset type" }).locator("..").locator("select").selectOption("polygon");
    await page.getByRole("button", { name: "Apply preset" }).click();
    await expect(page.getByText("Anchor 6", { exact: true })).toBeVisible({ timeout: 10_000 });

    await page.locator('input[type="file"][accept*="toml"]').setInputFiles({
      name: "venue.toml",
      mimeType: "text/plain",
      buffer: Buffer.from(toml),
    });

    await expect(page.getByText("Anchor 4", { exact: true })).toBeVisible({ timeout: 10_000 });
    await expect(page.getByText("Anchor 6", { exact: true })).toHaveCount(0);

    const venueAfter = await page.request.get("/venue");
    expect(venueAfter.ok()).toBeTruthy();
    const { classify: classifyAfter, venue: venueAfterData } = (await venueAfter.json()) as {
      classify: string;
      venue: { anchors: unknown[] };
    };
    expect(venueAfterData.anchors).toHaveLength(4);
    expect(classifyAfter).toBe(classifyBefore);
  });
});
