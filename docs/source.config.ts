import { rehypeCodeDefaultOptions } from "fumadocs-core/mdx-plugins";
import {
  defineConfig,
  defineDocs,
  frontmatterSchema,
} from "fumadocs-mdx/config";
import { transformerTwoslash } from "fumadocs-twoslash";
import { createFileSystemTypesCache } from "fumadocs-twoslash/cache-fs";
import z from "zod";

export const docs = defineDocs({
  dir: "content/docs",
  docs: {
    schema: frontmatterSchema.extend({
      index: z.boolean().default(false),
    }),
    postprocess: {
      includeProcessedMarkdown: true,
    },
  },
});

export default defineConfig({
  lastModifiedTime: "git",
  mdxOptions: {
    rehypeCodeOptions: {
      themes: {
        light: "github-light",
        dark: "github-dark",
      },
      langs: ["ts", "tsx", "js"],
      transformers: [
        ...(rehypeCodeDefaultOptions.transformers ?? []),
        transformerTwoslash({
          typesCache: createFileSystemTypesCache({
            dir: ".react-router/twoslash",
            cwd: process.cwd(),
          }),
        }),
      ],
    },
  },
});
