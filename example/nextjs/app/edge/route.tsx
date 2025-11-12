import { ImageResponse, initWasmSync } from "@takumi-rs/image-response/wasm";
import DocsTemplateV1 from "@takumi-rs/template/docs-template-v1";
import module from "@takumi-rs/wasm/next";
import { Axe } from "lucide-react";

initWasmSync(module);

export const runtime = "edge";

const fonts = [
  {
    name: "Geist",
    data: await fetch("https://takumi.kane.tw/fonts/Geist.woff2").then((r) =>
      r.arrayBuffer(),
    ),
  },
];

export function GET(request: Request) {
  const url = new URL(request.url);
  const name = url.searchParams.get("name") || "Takumi";

  return new ImageResponse(
    <DocsTemplateV1
      title={`Hello from ${name}!`}
      description="Try change the ?name parameter to see the change."
      icon={<Axe color="hsl(354, 90%, 60%)" size={64} />}
      primaryColor="hsla(354, 90%, 54%, 0.3)"
      primaryTextColor="hsl(354, 90%, 60%)"
      site="Takumi"
    />,
    {
      width: 1200,
      height: 630,
      format: "webp",
      fonts,
    },
  );
}
