import initWasm, { Renderer } from "@takumi-rs/wasm";
import wasmUrl from "@takumi-rs/wasm/takumi_wasm_bg.wasm?url";

let renderer: Renderer | undefined;

initWasm({ module_or_path: wasmUrl }).then(async () => {
  const font = await fetch("/fonts/Geist.woff2").then((r) => r.arrayBuffer());

  renderer = new Renderer();
  renderer.loadFont(new Uint8Array(font));

  self.postMessage({ type: "ready" });
});

self.onmessage = (event: MessageEvent) => {
  const { type, node } = event.data;

  if (type === "render" && renderer) {
    try {
      const start = performance.now();
      const dataUrl = renderer.renderAsDataUrl(node, 1200, 630, "png");
      const duration = performance.now() - start;

      self.postMessage({
        type: "render_complete",
        dataUrl,
        duration,
      });
    } catch (error) {
      self.postMessage({
        type: "render_error",
        error: error instanceof Error ? error.message : "Unknown error",
      });
    }
  }
};
