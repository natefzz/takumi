import { HomeLayout } from "fumadocs-ui/layouts/home";
import { Link } from "react-router";
import { Button } from "~/components/ui/button";
import { baseOptions } from "~/layout-config";

export function meta() {
  return [
    { title: "Takumi: Craft Beautiful Images with Code" },
    {
      name: "description",
      content:
        "A library for generating images using CSS Flexbox layout. Available for Rust, Node.js, and WebAssembly.",
    },
  ];
}

export default function Home() {
  return (
    <HomeLayout className="text-center" {...baseOptions}>
      <head>
        <meta
          name="og:image"
          content="https://raw.githubusercontent.com/kane50613/takumi/master/example/twitter-images/output/og-image.png"
        />
        <link rel="canonical" href="https://takumi.kane.tw/" />
      </head>
      <div className="max-w-5xl w-full mx-auto">
        <div className="flex flex-col py-24 px-4 items-center justify-center">
          <img src="/logo.svg" className="w-16 h-auto" alt="Takumi Logo" />
          <h1 className="py-6 text-3xl sm:text-5xl font-semibold max-w-4xl text-balance">
            <span className="text-primary">Takumi</span> makes dynamic image
            rendering simple.
          </h1>
          <p className="text-muted-foreground text-base sm:text-lg max-w-md mb-8">
            Production-ready library to make rendering performant, portable and
            scalable.
          </p>
          <div className="flex gap-2.5 mb-24">
            <Button asChild className="rounded-full" size="lg">
              <Link to="/docs">Open Docs</Link>
            </Button>
            <Button
              asChild
              className="rounded-full"
              variant="outline"
              size="lg"
            >
              <Link to="/playground">Try in Playground</Link>
            </Button>
          </div>
        </div>
      </div>
    </HomeLayout>
  );
}
