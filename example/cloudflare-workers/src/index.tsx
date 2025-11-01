import ImageResponse from "@takumi-rs/image-response/wasm";
import DocsTemplateV1 from "@takumi-rs/template/docs-template-v1";
import { initSync } from "@takumi-rs/wasm";
import module from "@takumi-rs/wasm/takumi_wasm_bg.wasm";
import geist from "../../../assets/fonts/geist/Geist[wght].woff2";
import { fetchLogo } from "./utils";

initSync({ module });

let logo: string;

export default {
  async fetch(request) {
    logo ??= await fetchLogo();

    const { searchParams } = new URL(request.url);

    const name = searchParams.get("name") || "Wizard";

    return new ImageResponse(
      <DocsTemplateV1
        title={`Hello, ${name}`}
        description="This is an example of rendering on Cloudflare Workers!"
        icon={
          <img
            src={logo}
            alt="Logo"
            style={{
              width: "6rem",
              borderRadius: "50%",
            }}
          />
        }
        site="Takumi"
        primaryColor="#F48120"
        primaryTextColor="#fff"
      />,
      {
        width: 1200,
        height: 630,
        format: "webp",
        fonts: [geist],
      },
    );
  },
} satisfies ExportedHandler<Env>;
