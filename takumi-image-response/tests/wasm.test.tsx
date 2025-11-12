import { describe, expect, test } from "bun:test";
import { ImageResponse, initWasm } from "../src/backends/wasm";

await initWasm(
  new URL(
    import.meta.resolve("@takumi-rs/wasm/takumi_wasm_bg.wasm"),
    import.meta.url,
  ),
);

describe("ImageResponse", () => {
  test("should not crash", async () => {
    const response = new ImageResponse(<div tw="bg-black w-4 h-4" />);

    expect(response.status).toBe(200);
    expect(response.headers.get("content-type")).toBe("image/webp");

    expect(await response.arrayBuffer()).toBeDefined();
  });

  test("should set content-type", async () => {
    const response = new ImageResponse(<div tw="bg-black w-4 h-4" />, {
      width: 100,
      height: 100,
      format: "png",
    });

    expect(response.headers.get("content-type")).toBe("image/png");
    expect(await response.arrayBuffer()).toBeDefined();
  });
});
