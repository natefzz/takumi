import {
  getPageTreePeers,
  type PageTree,
  type TOCItemType,
} from "fumadocs-core/server";
import { toClientRenderer } from "fumadocs-mdx/runtime/vite";
import { Card, Cards } from "fumadocs-ui/components/card";
import { Tab, Tabs } from "fumadocs-ui/components/tabs";
import { DocsLayout } from "fumadocs-ui/layouts/docs";
import defaultMdxComponents from "fumadocs-ui/mdx";
import {
  DocsBody,
  DocsDescription,
  DocsPage,
  DocsTitle,
} from "fumadocs-ui/page";
import { ArrowBigRight, BookOpen, Hand, Shovel } from "lucide-react";
import { redirect } from "react-router";
import { docs } from "source.generated";
import { baseOptions } from "~/layout-config";
import { source } from "~/source";
import type { Route } from "./+types/page";

const components = {
  ...defaultMdxComponents,
  Hand,
  BookOpen,
  ArrowBigRight,
  Shovel,
  DocsCategory,
  Tabs,
  Tab,
};

export function loader({ params }: Route.LoaderArgs) {
  const slugs = params["*"].split("/").filter((v) => v.length > 0);
  const page = source.getPage(slugs);

  if (!page) throw redirect("/docs");

  return {
    tree: source.pageTree,
    page,
    slugs,
  };
}

const clientLoader = toClientRenderer(docs.doc, ({ default: Mdx }) => (
  <Mdx components={components} />
));

export default function Page(props: Route.ComponentProps) {
  const { page, slugs, tree } = props.loaderData;

  const title = `${page.data.title} - Takumi`;

  const og = ["https://takumi.kane.tw/og", "docs", ...slugs, "image.webp"].join(
    "/",
  );

  const Content = clientLoader[page.path];

  return (
    <DocsLayout
      {...baseOptions}
      links={[
        {
          icon: <Shovel />,
          text: "Try in Playground",
          url: "/playground",
        },
      ]}
      tree={tree as PageTree.Root}
    >
      <DocsPage
        toc={page.data.toc as TOCItemType[]}
        lastUpdate={page.data.lastModified}
        editOnGithub={{
          owner: "kane50613",
          repo: "takumi",
          sha: "master",
          path: `/docs/content/docs/${page.path}?plain=1`,
        }}
      >
        <title>{title}</title>
        <meta name="description" content={page.data.description} />
        <meta name="og:title" content={title} />
        <meta name="og:description" content={page.data.description} />
        <meta name="og:image" content={og} />
        <meta name="twitter:image" content={og} />
        <DocsTitle>{page.data.title}</DocsTitle>
        <DocsDescription>{page.data.description}</DocsDescription>
        <DocsBody>
          <Content />
          {page.data.index ? (
            <DocsCategory tree={tree as PageTree.Root} url={page.url} />
          ) : null}
        </DocsBody>
      </DocsPage>
    </DocsLayout>
  );
}

function DocsCategory({ tree, url }: { tree: PageTree.Root; url: string }) {
  return (
    <Cards>
      {getPageTreePeers(tree, url).map((peer) => (
        <Card key={peer.url} title={peer.name} href={peer.url}>
          {peer.description}
        </Card>
      ))}
    </Cards>
  );
}
