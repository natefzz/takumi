import { expect, test } from "bun:test";
import { Renderer } from "../index";

const renderer = new Renderer();

test("report deserialize error with wrong type", () => {
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
    "InvalidArg, invalid type: integer `123`, expected a string like 'start', 'flex-start', 'center' or 'space-between'; also accepts 'initial' or 'inherit'",
  );
});

test("report deserialize error with invalid string value", () => {
  expect(() =>
    renderer.render(
      {
        type: "container",
        children: [],
        style: {
          justifyContent: "star",
        },
      },
      {
        width: 100,
        height: 100,
      },
    ),
  ).toThrowError(
    "InvalidArg, invalid value 'star', expected a string like 'start', 'flex-start', 'center' or 'space-between'",
  );
});
