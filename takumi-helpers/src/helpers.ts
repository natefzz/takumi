import type { CSSProperties } from "react";
import type { AnyNode, ContainerNode, ImageNode, TextNode } from "./types";

function applyStyle(node: AnyNode, style?: CSSProperties) {
  if (style && Object.keys(style).length > 0) {
    node.style = style;
  }
}

export function container(props: Omit<ContainerNode, "type">): ContainerNode {
  const node: ContainerNode = {
    type: "container",
    children: props.children,
    tw: props.tw,
  };

  applyStyle(node, props.style);

  return node;
}

export function text(text: string, style?: CSSProperties): TextNode;
export function text(props: Omit<TextNode, "type">): TextNode;

export function text(
  props: Omit<TextNode, "type"> | string,
  style?: CSSProperties,
): TextNode {
  if (typeof props === "string") {
    const node: TextNode = {
      type: "text",
      text: props,
    };

    applyStyle(node, style);

    return node;
  }

  const node: TextNode = {
    type: "text",
    text: props.text,
    tw: props.tw,
  };

  applyStyle(node, style ?? props.style);

  return node;
}

export function image(props: Omit<ImageNode, "type">): ImageNode {
  const node: ImageNode = {
    type: "image",
    src: props.src,
    width: props.width,
    height: props.height,
    tw: props.tw,
  };

  applyStyle(node, props.style);

  return node;
}

export function style(style: CSSProperties) {
  return style;
}

export function percentage(percentage: number) {
  return `${percentage}%` as const;
}

export function vw(vw: number) {
  return `${vw}vw` as const;
}

export function vh(vh: number) {
  return `${vh}vh` as const;
}

export function em(em: number) {
  return `${em}em` as const;
}

export function rem(rem: number) {
  return `${rem}rem` as const;
}

export function fr(fr: number) {
  return `${fr}fr` as const;
}

export function rgba(r: number, g: number, b: number, a = 1) {
  return `rgb(${r} ${g} ${b} / ${a})` as const;
}
