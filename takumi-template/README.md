# @takumi-rs/template

Template library for Takumi, it provides some pre-defined templates primarily for Takumi, but it could also be used in other libraries like satori.

## Usage

```tsx
import DocsTemplateV1 from "@takumi-rs/template/docs-template-v1";
import { fromJsx } from "@takumi-rs/helpers/jsx";
import { Renderer } from "@takumi-rs/core";

const render = new Renderer();

const node = await fromJsx(<DocsTemplateV1 />);
const buffer = await render.render(node, {
  width: 1200,
  height: 630,
  format: "webp",
});
```
