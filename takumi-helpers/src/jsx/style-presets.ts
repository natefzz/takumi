import type { JSX } from "react";
import type { PartialStyle } from "../types";

// Reference from Chromium's default style presets
// https://chromium.googlesource.com/chromium/blink/+/master/Source/core/css/html.css
// https://github.com/vercel/satori/blob/main/src/handler/presets.ts
export const stylePresets: Partial<
  Record<keyof JSX.IntrinsicElements, PartialStyle>
> = {
  // Generic block-level elements
  p: {
    marginTop: "1em",
    marginBottom: "1em",
  },
  blockquote: {
    marginTop: "1em",
    marginBottom: "1em",
    marginLeft: 40,
    marginRight: 40,
  },
  center: {
    textAlign: "center",
  },
  hr: {
    marginTop: "0.5em",
    marginBottom: "0.5em",
    marginLeft: "auto",
    marginRight: "auto",
    borderWidth: 1,
  },
  // Heading elements
  h1: {
    fontSize: "2em",
    marginTop: "0.67em",
    marginBottom: "0.67em",
    marginLeft: 0,
    marginRight: 0,
    fontWeight: "bold",
  },
  h2: {
    fontSize: "1.5em",
    marginTop: "0.83em",
    marginBottom: "0.83em",
    marginLeft: 0,
    marginRight: 0,
    fontWeight: "bold",
  },
  h3: {
    fontSize: "1.17em",
    marginTop: "1em",
    marginBottom: "1em",
    marginLeft: 0,
    marginRight: 0,
    fontWeight: "bold",
  },
  h4: {
    marginTop: "1.33em",
    marginBottom: "1.33em",
    marginLeft: 0,
    marginRight: 0,
    fontWeight: "bold",
  },
  h5: {
    fontSize: "0.83em",
    marginTop: "1.67em",
    marginBottom: "1.67em",
    marginLeft: 0,
    marginRight: 0,
    fontWeight: "bold",
  },
  h6: {
    fontSize: "0.67em",
    marginTop: "2.33em",
    marginBottom: "2.33em",
    marginLeft: 0,
    marginRight: 0,
    fontWeight: "bold",
  },
  u: {
    textDecoration: "underline",
  },
  strong: {
    fontWeight: "bold",
  },
  b: {
    fontWeight: "bold",
  },
  i: {
    fontStyle: "italic",
  },
  em: {
    fontStyle: "italic",
  },
  code: {
    fontFamily: "monospace",
  },
  kbd: {
    fontFamily: "monospace",
  },
  pre: {
    fontFamily: "monospace",
    marginTop: "1em",
    marginBottom: "1em",
  },
  mark: {
    backgroundColor: 0xffff00,
    color: 0,
  },
  big: {
    fontSize: "1.2em",
  },
  small: {
    fontSize: "0.8em",
  },
  s: {
    textDecoration: "line-through",
  },
};
