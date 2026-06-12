import { describe, expect, it } from "vitest";
import { cn, debounce, formatBytes, formatTimeAgo, relativeDateGroup } from "@/lib/utils";

describe("cn", () => {
  it("merges tailwind class lists", () => {
    expect(cn("px-2", "px-4")).toBe("px-4");
    expect(cn("text-red-500", false && "text-blue-500", "text-green-500")).toBe(
      "text-green-500"
    );
  });
});

describe("formatBytes", () => {
  it("formats bytes / KB / MB / GB", () => {
    expect(formatBytes(0)).toBe("0 B");
    expect(formatBytes(512)).toBe("512.0 B");
    expect(formatBytes(1024)).toBe("1.0 KB");
    expect(formatBytes(1024 * 1024)).toBe("1.0 MB");
    expect(formatBytes(1024 * 1024 * 1024)).toBe("1.0 GB");
  });
  it("clamps negative or non-finite values to 0 B", () => {
    expect(formatBytes(-1)).toBe("0 B");
    expect(formatBytes(NaN)).toBe("0 B");
    expect(formatBytes(Infinity)).toBe("0 B");
  });
});

describe("formatTimeAgo", () => {
  it("returns 'just now' for very recent timestamps", () => {
    expect(formatTimeAgo(Date.now() - 1000)).toBe("just now");
  });
  it("returns minutes for sub-hour deltas", () => {
    expect(formatTimeAgo(Date.now() - 5 * 60_000)).toBe("5m ago");
  });
  it("returns empty string for invalid timestamps", () => {
    expect(formatTimeAgo(0)).toBe("");
    expect(formatTimeAgo(NaN)).toBe("");
    expect(formatTimeAgo(-1)).toBe("");
  });
  it("returns a localized date for future timestamps (clock skew)", () => {
    const future = Date.now() + 60_000;
    const result = formatTimeAgo(future);
    expect(result).not.toBe("just now");
    expect(result).not.toContain("-");
  });
});

describe("relativeDateGroup", () => {
  const now = Date.now();
  it.each([
    [now, "Today"],
    [now - 86_400_000, "Yesterday"],
    [now - 3 * 86_400_000, "Last Week"],
    [now - 14 * 86_400_000, "Last Month"],
    [now - 200 * 86_400_000, "Last Year"],
    [now - 400 * 86_400_000, "Older"],
  ])("maps timestamp %i to %s", (ts, expected) => {
    expect(relativeDateGroup(ts)).toBe(expected);
  });
  it("returns 'Older' for invalid timestamps", () => {
    expect(relativeDateGroup(0)).toBe("Older");
    expect(relativeDateGroup(NaN)).toBe("Older");
    expect(relativeDateGroup(-1)).toBe("Older");
  });
});

describe("debounce", () => {
  it("delays invocation and only fires once for rapid calls", async () => {
    let calls = 0;
    const fn = debounce(() => calls++, 50);
    fn();
    fn();
    fn();
    expect(calls).toBe(0);
    await new Promise((r) => setTimeout(r, 80));
    expect(calls).toBe(1);
  });
});
