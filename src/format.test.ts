import { describe, expect, it } from "vitest";
import { formatBytes } from "./format";

describe("formatBytes", () => {
  it("formata zero", () => expect(formatBytes(0)).toBe("0 B"));
  it("escolhe unidade legível", () => expect(formatBytes(1024 ** 3)).toContain("GB"));
});
