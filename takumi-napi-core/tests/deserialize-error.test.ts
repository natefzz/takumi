import { expect, test } from "bun:test";
import { Renderer } from "../index";

const renderer = new Renderer();

test("report deserialize error", () => {
  expect(() =>
    renderer.render(
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
    "InvalidArg, unexpected token: Number { has_sign: false, value: 123.0, int_value: Some(123) }",
  );
});
