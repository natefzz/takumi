import { HomeLayout } from "fumadocs-ui/layouts/home";
import { Link } from "react-router";
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
      </head>
      <div className="max-w-5xl w-full mx-auto">
        <div className="flex flex-col py-24 px-4 items-center justify-center">
          <img src="/logo.svg" className="w-16 h-auto" alt="Takumi Logo" />
          <h1 className="py-6 text-3xl sm:text-5xl font-semibold max-w-4xl text-balance">
            <span className="text-accent">Takumi</span> makes dynamic image
            rendering simple.
          </h1>
          <p className="text-fd-primary/75 text-lg max-w-md mb-8">
            Production-ready library to make rendering performant, portable
            scalable.
          </p>
          <div className="flex gap-2.5 mb-24">
            <Link
              className="text-sm bg-fd-primary text-fd-primary-foreground rounded-full font-medium px-4 py-2.5"
              to="/docs"
            >
              Open Docs
            </Link>
            <Link
              className="text-sm border rounded-full font-medium px-4 py-2.5"
              to="/playground"
            >
              Try in Playground
            </Link>
          </div>
        </div>
      </div>
    </HomeLayout>
  );
}
