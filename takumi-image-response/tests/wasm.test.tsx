import { describe, expect, test } from "bun:test";
import { join } from "node:path";
import { file } from "bun";
import { ImageResponse, initWasm } from "../src/backends/wasm";

await initWasm(
  new URL(
    import.meta.resolve("@takumi-rs/wasm/takumi_wasm_bg.wasm"),
    import.meta.url,
  ),
);

const geist = await file(
  join(import.meta.dirname, "../../assets/fonts/geist/Geist[wght].woff2"),
).arrayBuffer();

describe("ImageResponse", () => {
  test("should not crash", async () => {
    const response = new ImageResponse(<div tw="bg-black w-4 h-4" />);

    expect(response.status).toBe(200);
    expect(response.headers.get("content-type")).toBe("image/webp");

    expect(await response.arrayBuffer()).toBeDefined();
  });

  test("should set content-type", async () => {
    const response = new ImageResponse(
      <div tw="bg-black w-4 h-4 text-white">Hello</div>,
      {
        width: 100,
        height: 100,
        format: "png",
        fonts: [
          {
            data: geist,
            name: "Geist",
          },
        ],
      },
    );

    expect(response.headers.get("content-type")).toBe("image/png");
    expect(await response.arrayBuffer()).toBeDefined();
  });
});
