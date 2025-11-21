import { describe, expect, test } from "bun:test";
import { readFile } from "node:fs/promises";
import { join } from "node:path";
import { container, image, text } from "@takumi-rs/helpers";
import { Glob } from "bun";
import init, { AnimationFrameSource, Renderer } from "../pkg/takumi_wasm";

await init({
  module_or_path: readFile("./pkg/takumi_wasm_bg.wasm"),
});

const fontsGlob = new Glob("**/*.{woff2,ttf}");

async function getFonts() {
  const fonts: Buffer[] = [];

  for await (const file of fontsGlob.scan("../assets/fonts")) {
    fonts.push(await readFile(join("../assets/fonts", file)));
  }

  return fonts;
}

const fonts = await getFonts();
const renderer = new Renderer();

const localImagePath = "../assets/images/yeecord.png";

const localImage = await readFile(localImagePath);
const dataUri = `data:image/png;base64,${Buffer.from(localImage).toString(
  "base64",
)}`;

const node = container({
  children: [
    image({
      src: dataUri,
      width: 96,
      height: 96,
      style: {
        borderRadius: "25%",
      },
    }),
    text("Data URI"),
  ],
  style: {
    justifyContent: "center",
    alignItems: "center",
    gap: "1.5rem",
    fontSize: "1.5rem",
    backgroundColor: "white",
    width: "100%",
    height: "100%",
  },
});

describe("setup", () => {
  test(`loadFonts (${fonts.length})`, () => {
    for (const font of fonts) renderer.loadFont(font);
  });

  test("putPersistentImage", () => {
    renderer.putPersistentImage(localImagePath, new Uint8Array(localImage));
  });
});

describe("render", () => {
  test("webp", () => {
    const result = renderer.render(node, {
      width: 1200,
      height: 630,
      format: "webp",
    });

    expect(result).toBeInstanceOf(Uint8Array);
  });

  test("png", () => {
    const result = renderer.render(node, {
      width: 1200,
      height: 630,
      format: "png",
    });

    expect(result).toBeInstanceOf(Uint8Array);
  });

  test("jpeg 75%", () => {
    const result = renderer.render(node, {
      width: 1200,
      height: 630,
      format: "jpeg",
      quality: 75,
    });

    expect(result).toBeInstanceOf(Uint8Array);
  });

  test("jpeg 100%", () => {
    const result = renderer.render(node, {
      width: 1200,
      height: 630,
      format: "jpeg",
      quality: 100,
    });

    expect(result).toBeInstanceOf(Uint8Array);
  });
});

describe("renderAsDataUrl", () => {
  test("default format (png)", () => {
    const result = renderer.renderAsDataUrl(node, { width: 1200, height: 630 });

    expect(result).toMatch(/^data:image\/png;base64,/);
    expect(result.length).toBeGreaterThan(100);
  });

  test("webp format", () => {
    const result = renderer.renderAsDataUrl(node, {
      width: 1200,
      height: 630,
      format: "webp",
    });

    expect(result).toMatch(/^data:image\/webp;base64,/);
    expect(result.length).toBeGreaterThan(100);
  });

  test("jpeg format with quality", () => {
    const result = renderer.renderAsDataUrl(node, {
      width: 1200,
      height: 630,
      format: "jpeg",
      quality: 75,
    });

    expect(result).toMatch(/^data:image\/jpeg;base64,/);
    expect(result.length).toBeGreaterThan(100);
  });

  test("png format explicit", () => {
    const result = renderer.renderAsDataUrl(node, {
      width: 1200,
      height: 630,
      format: "png",
    });

    expect(result).toMatch(/^data:image\/png;base64,/);
    expect(result.length).toBeGreaterThan(100);
  });

  describe("renderAnimation", () => {
    test("webp", () => {
      const frame = new AnimationFrameSource(node, 1000);
      const result = renderer.renderAnimation([frame], {
        width: 1200,
        height: 630,
        format: "webp",
      });

      expect(result).toBeInstanceOf(Uint8Array);
    });

    test("apng", () => {
      const frame = new AnimationFrameSource(node, 1000);
      const result = renderer.renderAnimation([frame], {
        width: 1200,
        height: 630,
        format: "apng",
      });

      expect(result).toBeInstanceOf(Uint8Array);
    });
  });
});
