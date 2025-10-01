import { join } from "node:path";
import { Renderer } from "@takumi-rs/core";
import { fromJsx } from "@takumi-rs/helpers/jsx";
import { file, write } from "bun";
import * as FiveHundredStars from "./components/500-stars";
import * as OgImage from "./components/og-image";

const components = [OgImage, FiveHundredStars];

type Component = (typeof components)[number];

async function render(module: Component) {
  const component = await fromJsx(<module.default />);

  const renderer = new Renderer({
    persistentImages: module.persistentImages,
    fonts: await Promise.all(
      module.fonts.map((font) =>
        file(join("../../assets/fonts", font)).arrayBuffer(),
      ),
    ),
  });

  const start = performance.now();

  const buffer = await renderer.render(component, {
    width: module.width,
    height: module.height,
  });

  const end = performance.now();

  await write(join("output", `${module.name}.png`), buffer);

  console.log(`Rendered ${module.name} in ${Math.round(end - start)}ms`);
}

for (const component of components) {
  await render(component);
}
