import { describe, expect, it } from "vitest";
import { captureDate, formatBytes } from "./format";

describe("formatBytes", () => {
  it("formata zero", () => expect(formatBytes(0)).toBe("0 B"));
  it("escolhe unidade legível", () => expect(formatBytes(1024 ** 3)).toContain("GB"));
});

describe("horário de captura", () => {
  it("preserva a hora local gravada pela câmera", () => {
    const value = captureDate("2026-01-02T14:30:00");
    expect(value.getFullYear()).toBe(2026);
    expect(value.getMonth()).toBe(0);
    expect(value.getDate()).toBe(2);
    expect(value.getHours()).toBe(14);
    expect(value.getMinutes()).toBe(30);
  });

  it("não desloca uma data sem horário para o dia anterior", () => {
    const value = captureDate("2026-01-02");
    expect(value.getDate()).toBe(2);
    expect(value.getHours()).toBe(0);
  });
});
