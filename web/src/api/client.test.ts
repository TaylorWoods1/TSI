import { describe, expect, it, vi, beforeEach, afterEach } from "vitest";

describe("api client error handling", () => {
  beforeEach(() => {
    vi.stubGlobal("fetch", vi.fn());
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("parses JSON error bodies", async () => {
    const { health } = await import("./client");
    (fetch as ReturnType<typeof vi.fn>).mockResolvedValueOnce({
      ok: false,
      text: async () => JSON.stringify({ error: "bad venue" }),
    });
    await expect(health()).rejects.toThrow("bad venue");
  });

  it("health returns version on success", async () => {
    const { health } = await import("./client");
    (fetch as ReturnType<typeof vi.fn>).mockResolvedValueOnce({
      ok: true,
      json: async () => ({ ok: true, version: "0.1.0" }),
    });
    const h = await health();
    expect(h.version).toBe("0.1.0");
  });
});
