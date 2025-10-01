import { expect, test } from "bun:test";
import { Renderer } from "../index";

const renderer = new Renderer();

test("report deserialize error", () => {
  expect(() =>
    renderer.renderSync(
      {
        type: "container",
        children: [],
        style: {
          justifyContent: 123,
        },
      },
      {
        width: 100,
        height: 100,
      },
    ),
  ).toThrowError(
    "InvalidArg, invalid type: integer `123`, expected enum JustifyContent",
  );
});
