import type { CSSProperties } from "react";

export type AnyNode = {
  type: string;
  style?: CSSProperties;
  tw?: string;
  [key: string]: unknown;
};

/**
 * @deprecated Use {import("csstype").Properties} or {import("react").CSSProperties} instead
 */
export type PartialStyle = CSSProperties;

export type Node = ContainerNode | TextNode | ImageNode | AnyNode;

export type ContainerNode = {
  type: "container";
  style?: CSSProperties;
  children?: Node[];
  tw?: string;
};

export type TextNode = {
  type: "text";
  text: string;
  style?: CSSProperties;
  tw?: string;
};

export type ImageNode = {
  type: "image";
  src: string;
  width?: number;
  height?: number;
  style?: CSSProperties;
  tw?: string;
};
