import type { Config } from "@react-router/dev/config";

function getDocsPaths() {
  const paths = Object.keys(
    import.meta.glob("./**/*.{mdx,md}", {
      base: "./content",
    }),
  );

  return paths.map(relativeFileNameToPath);
}

function relativeFileNameToPath(name: string) {
  const path = name.slice(1).replace(/(index)?\.(md|mdx)$/, "");

  if (!path.endsWith("/")) return `${path}/`;

  return path;
}

export default {
  prerender({ getStaticPaths }) {
    const docsPaths = getDocsPaths();
    const docsOgPaths = docsPaths.map((path) => `/og${path}image.webp`);

    return [...getStaticPaths(), ...docsPaths, ...docsOgPaths];
  },
  routeDiscovery: {
    mode: "initial",
  },
} satisfies Config;
