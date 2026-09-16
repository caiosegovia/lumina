import { describe, expect, it } from "vitest";
import { BoundedLru } from "./lru";

describe("BoundedLru", () => {
  it("evicts the least recently used entry", () => {
    const cache = new BoundedLru<string, number>(2);
    cache.set("a", 1);
    cache.set("b", 2);
    expect(cache.get("a")).toBe(1);
    cache.set("c", 3);
    expect(cache.get("b")).toBeUndefined();
    expect(cache.get("a")).toBe(1);
    expect(cache.get("c")).toBe(3);
    expect(cache.size).toBe(2);
  });
});
