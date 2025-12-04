use takumi::layout::{
  node::{ContainerNode, ImageNode, NodeKind, TextNode},
  style::{PercentageNumber, *},
};

use crate::test_utils::run_style_width_test;

mod test_utils;

fn create_test_container(opacity: f32) -> NodeKind {
  ContainerNode {
    tw: None,
    style: Some(
      StyleBuilder::default()
        .width(LengthUnit::Percentage(8.0))
        .height(LengthUnit::Percentage(6.0))
        .border_radius(BorderRadius(Sides(
          [SpacePair::from_single(LengthUnit::Rem(1.0)); 4],
        )))
        .opacity(PercentageNumber(opacity))
        .justify_content(JustifyContent::Center)
        .align_items(AlignItems::Center)
        .background_color(ColorInput::Value(Color([255, 0, 0, 255])))
        .build()
        .unwrap(),
    ),
    children: Some(vec![
      TextNode {
        tw: None,
        style: None,
        text: opacity.to_string(),
      }
      .into(),
    ]),
  }
  .into()
}

#[test]
fn test_style_opacity() {
  let container = ContainerNode {
    tw: None,
    style: Some(
      StyleBuilder::default()
        .width(LengthUnit::Percentage(100.0))
        .height(LengthUnit::Percentage(100.0))
        .justify_content(JustifyContent::Center)
        .align_items(AlignItems::Center)
        .background_color(ColorInput::Value(Color([255, 255, 255, 255])))
        .gap(SpacePair::from_single(LengthUnit::Rem(4.0)))
        .build()
        .unwrap(),
    ),
    children: Some(vec![
      create_test_container(0.1),
      create_test_container(0.3),
      create_test_container(0.5),
      create_test_container(1.0),
    ]),
  };

  run_style_width_test(container.into(), "tests/fixtures/style_opacity.webp");
}

#[test]
fn test_style_opacity_image_with_text() {
  let container = ContainerNode {
    tw: None,
    style: Some(
      StyleBuilder::default()
        .width(LengthUnit::Percentage(100.0))
        .height(LengthUnit::Percentage(100.0))
        .justify_content(JustifyContent::Center)
        .align_items(AlignItems::Center)
        .flex_direction(FlexDirection::Column)
        .gap(SpacePair::from_single(LengthUnit::Rem(2.0)))
        .background_color(ColorInput::Value(Color([240, 240, 240, 255])))
        .build()
        .unwrap(),
    ),
    children: Some(vec![
      ContainerNode {
        tw: None,
        style: Some(
          StyleBuilder::default()
            .width(LengthUnit::Rem(20.0))
            .height(LengthUnit::Rem(20.0))
            .opacity(PercentageNumber(0.5))
            .build()
            .unwrap(),
        ),
        children: Some(vec![
          ImageNode {
            tw: None,
            style: Some(
              StyleBuilder::default()
                .width(LengthUnit::Percentage(100.0))
                .height(LengthUnit::Percentage(100.0))
                .build()
                .unwrap(),
            ),
            src: "assets/images/yeecord.png".into(),
            width: None,
            height: None,
          }
          .into(),
        ]),
      }
      .into(),
      TextNode {
        tw: None,
        style: Some(
          StyleBuilder::default()
            .font_size(Some(LengthUnit::Rem(3.0)))
            .font_weight(FontWeight::from(700.0))
            .color(ColorInput::Value(Color([60, 60, 60, 255])))
            .opacity(PercentageNumber(0.5))
            .build()
            .unwrap(),
        ),
        text: "0.5".to_string(),
      }
      .into(),
    ]),
  };

  run_style_width_test(
    container.into(),
    "tests/fixtures/style_opacity_image_with_text.webp",
  );
}
