import { loader } from "fumadocs-core/source";
import defaultAttributes from "lucide-react/dist/esm/defaultAttributes";
import { __iconNode as arrowBigRightIconNode } from "lucide-react/dist/esm/icons/arrow-big-right";
import { __iconNode as axeIconNode } from "lucide-react/dist/esm/icons/axe";
import { __iconNode as bookIconNode } from "lucide-react/dist/esm/icons/book";
import { __iconNode as brainIconNode } from "lucide-react/dist/esm/icons/brain";
import { __iconNode as flaskConicalIconNode } from "lucide-react/dist/esm/icons/flask-conical";
import { __iconNode as leafIconNode } from "lucide-react/dist/esm/icons/leaf";
import { createElement } from "react";
import { create, docs } from "source.generated";

const iconProps = {
  ...defaultAttributes,
  color: "currentColor",
  strokeWidth: 2,
  width: 24,
  height: 24,
};

const icons = {
  Leaf: leafIconNode,
  Brain: brainIconNode,
  Book: bookIconNode,
  FlaskConical: flaskConicalIconNode,
  Axe: axeIconNode,
  ArrowBigRight: arrowBigRightIconNode,
};

export const source = loader({
  icon(icon) {
    if (icon && icon in icons) {
      return createElement(
        "svg",
        iconProps,
        icons[icon as keyof typeof icons].map(([tag, attrs]) =>
          createElement(tag, attrs),
        ),
      );
    }
  },
  source: await create.sourceAsync(docs.doc, docs.meta),
  baseUrl: "/docs",
});
