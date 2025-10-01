import { test } from "bun:test";
import { join } from "node:path";
import { Renderer } from "@takumi-rs/core";
import { fromJsx } from "@takumi-rs/helpers/jsx";
import { write } from "bun";
import type { ReactNode } from "react";
import DocsTemplateV1 from "../src/templates/docs-template-v1";

const renderer = new Renderer({
  persistentImages: [
    {
      src: "takumi.svg",
      data: await Bun.file(
        join(import.meta.dirname, "..", "..", "assets", "images", "takumi.svg"),
      ).arrayBuffer(),
    },
  ],
});

function testRender(name: string, template: ReactNode) {
  test(name, async () => {
    const node = await fromJsx(template);
    const start = performance.now();

    const buffer = await renderer.render(node, {
      width: 1200,
      height: 630,
      format: "webp",
    });

    const end = performance.now();

    console.log(`Rendered in ${Math.round(end - start)}ms`);

    await write(join(import.meta.dirname, "output", `${name}.webp`), buffer);
  });
}

testRender(
  "DocsTemplateV1",
  <DocsTemplateV1
    title="Fumadocs Integration"
    description="When will Fuma meet me in person? Hope we can meet in Japan! Culpa dolore eu ullamco aute exercitation sint aute nostrud qui tempor commodo ad culpa culpa. Laborum laboris eu laborum Lorem aliquip nulla nulla est proident eu. Officia deserunt aute ex quis exercitation ut. Irure cupidatat eu dolor Lorem eu aliquip mollit voluptate esse aute fugiat officia proident aliquip."
    icon={
      <img
        alt="Takumi"
        src="takumi.svg"
        style={{ width: "4rem", height: "4rem" }}
      />
    }
    primaryColor="hsla(354, 90%, 54%, 0.3)"
    primaryTextColor="hsl(354, 90%, 60%)"
    site="Takumi"
  />,
);
