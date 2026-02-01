import { describe, expect, it } from "vitest";
import { bytesToHexPreview, formatBytes } from "./bytes";

describe("bytesToHexPreview", () => {
  it("formats bytes as lower-case hex pairs", () => {
    const input = new Uint8Array([0x00, 0x09, 0x0a, 0x10, 0xff]);
    expect(bytesToHexPreview(input, 16)).toBe("00 09 0a 10 ff");
  });

  it("respects maxBytes", () => {
    const input = new Uint8Array([0x01, 0x02, 0x03]);
    expect(bytesToHexPreview(input, 2)).toBe("01 02");
  });
});

describe("formatBytes", () => {
  it("formats bytes in IEC units", () => {
    expect(formatBytes(0)).toBe("0 B");
    expect(formatBytes(1024)).toBe("1.00 KiB");
    expect(formatBytes(1024 * 1024)).toBe("1.00 MiB");
  });
});
