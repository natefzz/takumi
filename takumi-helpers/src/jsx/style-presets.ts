// Modified from https://github.com/vercel/satori/blob/2a0878a7f329bdba3a17ad68f71186a47add0dde/src/handler/presets.ts
// Reference from https://chromium.googlesource.com/chromium/blink/+/master/Source/core/css/html.css

import type { CSSProperties, JSX } from "react";

export const stylePresets: Partial<
  Record<keyof JSX.IntrinsicElements, CSSProperties>
> = {
  body: {
    margin: 8,
  },
  // Generic block-level elements
  p: {
    marginTop: "1em",
    marginBottom: "1em",
    display: "block",
  },
  blockquote: {
    marginTop: "1em",
    marginBottom: "1em",
    marginLeft: 40,
    marginRight: 40,
    display: "block",
  },
  center: {
    textAlign: "center",
    display: "block",
  },
  hr: {
    marginTop: "0.5em",
    marginBottom: "0.5em",
    marginLeft: "auto",
    marginRight: "auto",
    borderWidth: 1,
    display: "block",
  },
  // Heading elements
  h1: {
    fontSize: "2em",
    marginTop: "0.67em",
    marginBottom: "0.67em",
    marginLeft: 0,
    marginRight: 0,
    fontWeight: "bold",
    display: "block",
  },
  h2: {
    fontSize: "1.5em",
    marginTop: "0.83em",
    marginBottom: "0.83em",
    marginLeft: 0,
    marginRight: 0,
    fontWeight: "bold",
    display: "block",
  },
  h3: {
    fontSize: "1.17em",
    marginTop: "1em",
    marginBottom: "1em",
    marginLeft: 0,
    marginRight: 0,
    fontWeight: "bold",
    display: "block",
  },
  h4: {
    marginTop: "1.33em",
    marginBottom: "1.33em",
    marginLeft: 0,
    marginRight: 0,
    fontWeight: "bold",
    display: "block",
  },
  h5: {
    fontSize: "0.83em",
    marginTop: "1.67em",
    marginBottom: "1.67em",
    marginLeft: 0,
    marginRight: 0,
    fontWeight: "bold",
    display: "block",
  },
  h6: {
    fontSize: "0.67em",
    marginTop: "2.33em",
    marginBottom: "2.33em",
    marginLeft: 0,
    marginRight: 0,
    fontWeight: "bold",
    display: "block",
  },
  u: {
    textDecoration: "underline",
    display: "inline",
  },
  strong: {
    fontWeight: "bold",
    display: "inline",
  },
  b: {
    fontWeight: "bold",
    display: "inline",
  },
  i: {
    fontStyle: "italic",
    display: "inline",
  },
  em: {
    fontStyle: "italic",
    display: "inline",
  },
  code: {
    fontFamily: "monospace",
    display: "inline",
  },
  kbd: {
    fontFamily: "monospace",
    display: "inline",
  },
  pre: {
    fontFamily: "monospace",
    margin: "1em 0",
    display: "block",
  },
  mark: {
    backgroundColor: "yellow",
    color: "black",
    display: "inline",
  },
  big: {
    fontSize: "1.2em",
    display: "inline",
  },
  small: {
    fontSize: "0.8em",
    display: "inline",
  },
  s: {
    textDecoration: "line-through",
    display: "inline",
  },
  span: {
    display: "inline",
  },
  img: {
    display: "inline",
  },
  svg: {
    display: "inline",
  },
};
