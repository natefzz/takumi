import type { ColorInput } from "./bindings/ColorInput";
import type {
  AnyNode,
  ContainerNode,
  ImageNode,
  PartialStyle,
  TextNode,
} from "./types";

function applyStyle(node: AnyNode, style?: PartialStyle) {
  if (style && Object.keys(style).length > 0) {
    node.style = style;
  }
}

export function container(props: Omit<ContainerNode, "type">): ContainerNode {
  const node: ContainerNode = {
    type: "container",
    children: props.children,
  };

  applyStyle(node, props.style);

  return node;
}

export function text(text: string, style?: PartialStyle): TextNode {
  const node: TextNode = {
    type: "text",
    text,
  };

  applyStyle(node, style);

  return node;
}

export function image(props: Omit<ImageNode, "type">): ImageNode {
  const node: ImageNode = {
    type: "image",
    src: props.src,
  };

  applyStyle(node, props.style);

  return node;
}

export function style(style: PartialStyle) {
  return style;
}

/**
 * Convert a number to a percentage struct.
 * @param percentage - The percentage to convert (0.0 - 100.0).
 * @returns The percentage struct.
 */
export function percentage(percentage: number) {
  return {
    percentage,
  };
}

export function vw(vw: number) {
  return {
    vw,
  };
}

export function vh(vh: number) {
  return {
    vh,
  };
}

export function em(em: number) {
  return {
    em,
  };
}

export function rem(rem: number) {
  return {
    rem,
  };
}

export function fr(fr: number) {
  return {
    fr,
  };
}

export function rgba(r: number, g: number, b: number, a = 1): ColorInput {
  return [r, g, b, a];
}
