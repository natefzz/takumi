use smallvec::smallvec;
use takumi::layout::{
  node::{ContainerNode, TextNode},
  style::{
    BoxShadow, BoxShadows, Color, CssOption, FontWeight,
    LengthUnit::{Percentage, Px, Rem},
    LineHeight, Position, Sides, StyleBuilder,
  },
};

mod test_utils;
use test_utils::run_style_width_test;

#[test]
fn test_style_background_color() {
  let container = ContainerNode {
    style: StyleBuilder::default()
      .width(Percentage(100.0))
      .height(Percentage(100.0))
      .background_color(Color([255, 0, 0, 255]))
      .build()
      .unwrap(),
    children: None,
  };

  run_style_width_test(
    container.into(),
    "tests/fixtures/style_background_color.png",
  );
}

#[test]
fn test_style_border_radius() {
  let container = ContainerNode {
    style: StyleBuilder::default()
      .width(Percentage(100.0))
      .height(Percentage(100.0))
      .background_color(Color([255, 0, 0, 255]))
      .border_radius(Sides([Px(20.0); 4]))
      .build()
      .unwrap(),
    children: None,
  };

  run_style_width_test(container.into(), "tests/fixtures/style_border_radius.png");
}

#[test]
fn test_style_border_radius_per_corner() {
  let container = ContainerNode {
    style: StyleBuilder::default()
      .width(Percentage(100.0))
      .height(Percentage(100.0))
      .background_color(Color([255, 0, 0, 255]))
      // Per-corner radii
      .border_top_left_radius(CssOption::some(Px(40.0)))
      .border_top_right_radius(CssOption::some(Px(10.0)))
      .border_bottom_right_radius(CssOption::some(Px(80.0)))
      .border_bottom_left_radius(CssOption::some(Px(0.0)))
      .build()
      .unwrap(),
    children: None,
  };

  run_style_width_test(
    container.into(),
    "tests/fixtures/style_border_radius_per_corner.png",
  );
}

#[test]
fn test_style_border_width() {
  let container = ContainerNode {
    style: StyleBuilder::default()
      .width(Percentage(100.0))
      .height(Percentage(100.0))
      .background_color(Color::white())
      .border_width(CssOption::some(Sides([Px(10.0); 4])))
      .border_color(CssOption::some(Color([255, 0, 0, 255])))
      .build()
      .unwrap(),
    children: None,
  };

  run_style_width_test(container.into(), "tests/fixtures/style_border_width.png");
}

#[test]
fn test_style_border_width_with_radius() {
  let container = ContainerNode {
    style: StyleBuilder::default()
      .width(Percentage(100.0))
      .height(Percentage(100.0))
      .padding(Sides([Rem(4.0); 4]))
      .background_color(Color::white())
      .border_color(CssOption::some(Color([255, 0, 0, 255])))
      .build()
      .unwrap(),
    children: Some(vec![
      ContainerNode {
        style: StyleBuilder::default()
          .width(Rem(16.0))
          .height(Rem(8.0))
          .border_radius(Sides([Px(10.0); 4]))
          .border_width(CssOption::some(Sides([Px(4.0); 4])))
          .build()
          .unwrap(),
        children: None,
      }
      .into(),
    ]),
  };

  run_style_width_test(
    container.into(),
    "tests/fixtures/style_border_width_with_radius.png",
  );
}

#[test]
fn test_style_box_shadow() {
  let container = ContainerNode {
    style: StyleBuilder::default()
      .width(Percentage(100.0))
      .height(Percentage(100.0))
      .background_color(Color([0, 0, 255, 255]))
      .build()
      .unwrap(),
    children: Some(vec![
      ContainerNode {
        style: StyleBuilder::default()
          .width(Px(100.0))
          .height(Px(100.0))
          .background_color(Color([255, 0, 0, 255]))
          .box_shadow(CssOption::some(BoxShadows(smallvec![BoxShadow {
            color: Color([0, 0, 0, 128]),
            offset_x: Px(5.0),
            offset_y: Px(5.0),
            blur_radius: Px(10.0),
            spread_radius: Px(0.0),
            inset: false,
          }])))
          .build()
          .unwrap(),
        children: None,
      }
      .into(),
    ]),
  };

  run_style_width_test(container.into(), "tests/fixtures/style_box_shadow.png");
}

#[test]
fn test_style_box_shadow_inset() {
  let container = ContainerNode {
    style: StyleBuilder::default()
      .width(Percentage(100.0))
      .height(Percentage(100.0))
      .background_color(Color([0, 0, 255, 255])) // Blue background container for contrast
      .build()
      .unwrap(),
    children: Some(vec![
      ContainerNode {
        style: StyleBuilder::default()
          .width(Px(120.0))
          .height(Px(80.0))
          .background_color(Color::white()) // White child for inset visibility
          .border_radius(Sides([Px(16.0); 4]))
          .box_shadow(CssOption::some(BoxShadows(smallvec![BoxShadow {
            color: Color([0, 0, 0, 153]),
            offset_x: Px(4.0),
            offset_y: Px(6.0),
            blur_radius: Px(18.0),
            spread_radius: Px(8.0),
            inset: true,
          }])))
          .build()
          .unwrap(),
        children: None,
      }
      .into(),
    ]),
  };

  run_style_width_test(
    container.into(),
    "tests/fixtures/style_box_shadow_inset.png",
  );
}

#[test]
fn test_style_position() {
  let container = ContainerNode {
    style: StyleBuilder::default()
      .width(Percentage(100.0))
      .height(Percentage(100.0))
      .background_color(Color([0, 0, 255, 255]))
      .build()
      .unwrap(),
    children: Some(vec![
      ContainerNode {
        style: StyleBuilder::default()
          .width(Px(100.0))
          .height(Px(100.0))
          .position(Position::Absolute) // Test the position property
          .inset(Sides([Px(20.0); 4])) // Position with inset properties
          .background_color(Color([255, 0, 0, 255])) // Red child to make it visible
          .build()
          .unwrap(),
        children: None,
      }
      .into(),
    ]),
  };

  run_style_width_test(container.into(), "tests/fixtures/style_position.png");
}

#[test]
fn test_style_border_radius_circle() {
  let container = ContainerNode {
    style: StyleBuilder::default()
      .width(Px(300.0))
      .height(Px(300.0))
      .background_color(Color([255, 0, 0, 255]))
      .border_radius(Sides([Percentage(50.0); 4]))
      .build()
      .unwrap(),
    children: None,
  };

  run_style_width_test(
    container.into(),
    "tests/fixtures/style_border_radius_circle.png",
  );
}

// https://github.com/kane50613/takumi/issues/151
#[test]
fn test_style_border_radius_width_offset() {
  let container = ContainerNode {
    style: StyleBuilder::default()
      .width(Percentage(100.0))
      .height(Percentage(100.0))
      .background_color(Color([128, 128, 128, 255]))
      .padding(Sides([Rem(2.0); 4]))
      .build()
      .unwrap(),
    children: Some(vec![
      ContainerNode {
        style: StyleBuilder::default()
          .width(Percentage(100.0))
          .height(Percentage(100.0))
          .background_color(Color::white())
          .border_width(CssOption::some(Sides([Px(1.0); 4])))
          .border_radius(Sides([Px(24.0); 4]))
          .border_color(CssOption::some(Color([0, 0, 0, 255])))
          .build()
          .unwrap(),
        children: Some(vec![
          TextNode {
            text: "The newest blog post".to_string(),
            style: StyleBuilder::default()
              .width(Percentage(100.0))
              .padding(Sides([Rem(4.0); 4]))
              .font_size(CssOption::some(Rem(4.0)))
              .font_weight(FontWeight::from(500.0))
              .line_height(LineHeight(Rem(4.0 * 1.5)))
              .build()
              .unwrap(),
          }
          .into(),
        ]),
      }
      .into(),
    ]),
  };

  run_style_width_test(
    container.into(),
    "tests/fixtures/style_border_radius_width_offset.png",
  );
}
