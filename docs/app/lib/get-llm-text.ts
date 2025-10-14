import type { InferPageType } from "fumadocs-core/source";
import type { source } from "~/source";

export async function getLLMText(page: InferPageType<typeof source>) {
  return `# ${page.data.title}
URL: ${page.url}
${page.data.description}
${await page.data.getText("processed")}`;
}
