use takumi::layout::{
  node::{ContainerNode, NodeKind},
  style::{
    BackgroundImagesValue, BackgroundPositionsValue, BackgroundRepeat, BackgroundRepeats,
    BackgroundRepeatsValue, BackgroundSizesValue, Color, CssOption, LengthUnit::Percentage,
    StyleBuilder,
  },
};

mod test_utils;
use test_utils::run_style_width_test;

fn create_container(background_images: BackgroundImagesValue) -> ContainerNode<NodeKind> {
  ContainerNode {
    style: StyleBuilder::default()
      .width(Percentage(100.0))
      .height(Percentage(100.0))
      .background_image(CssOption::some(background_images.try_into().unwrap()))
      .build()
      .unwrap(),
    children: None,
  }
}

fn create_container_with(
  background_images: BackgroundImagesValue,
  background_size: Option<BackgroundSizesValue>,
  background_position: Option<BackgroundPositionsValue>,
  background_repeat: Option<BackgroundRepeatsValue>,
) -> ContainerNode<NodeKind> {
  ContainerNode {
    style: StyleBuilder::default()
      .width(Percentage(100.0))
      .height(Percentage(100.0))
      .background_image(CssOption::some(background_images.try_into().unwrap()))
      .background_size(CssOption(background_size.map(|v| v.try_into().unwrap())))
      .background_position(CssOption(
        background_position.map(|v| v.try_into().unwrap()),
      ))
      .background_repeat(CssOption(background_repeat.map(|v| v.try_into().unwrap())))
      .build()
      .unwrap(),
    children: None,
  }
}

#[test]
fn test_style_background_image_gradient_basic() {
  let background_images =
    BackgroundImagesValue::Css("linear-gradient(45deg, #007aff, #34c759)".to_string());

  let container = create_container(background_images);

  run_style_width_test(
    container.into(),
    "tests/fixtures/style_background_image_gradient.png",
  );
}

#[test]
fn test_style_background_image_gradient_alt() {
  let background_images =
    BackgroundImagesValue::Css("linear-gradient(0deg, #ff3b30, #5856d6)".to_string());

  let container = create_container(background_images);

  run_style_width_test(
    container.into(),
    "tests/fixtures/style_background_image_gradient_alt.png",
  );
}

#[test]
fn test_style_background_image_gradient_hard_stop() {
  let background_images =
    BackgroundImagesValue::Css("linear-gradient(to left, #252525 0%, #252525 20%, #f5f5f5 20%, #f5f5f5 40%, #00b7b7 40%, #00b7b7 60%, #b70000 60%, #b70000 80%, #fcd50e 80%)".to_string());

  let container = create_container(background_images);

  run_style_width_test(
    container.into(),
    "tests/fixtures/style_background_image_gradient_hard_stop.png",
  );
}

#[test]
fn test_style_background_image_radial_basic() {
  let background_images =
    BackgroundImagesValue::Css("radial-gradient(#e66465, #9198e5)".to_string());

  let container = create_container(background_images);

  run_style_width_test(
    container.into(),
    "tests/fixtures/style_background_image_radial_basic.png",
  );
}

#[test]
fn test_style_background_image_radial_mixed() {
  let background_images = BackgroundImagesValue::Css("radial-gradient(ellipse at top, #e66465, transparent), radial-gradient(ellipse at bottom, #4d9f0c, transparent)".to_string());

  let container = create_container(background_images);

  run_style_width_test(
    container.into(),
    "tests/fixtures/style_background_image_radial_mixed.png",
  );
}

#[test]
fn test_style_background_image_linear_radial_mixed() {
  let background_images = BackgroundImagesValue::Css(
    "linear-gradient(45deg, #0000ff, #00ff00), radial-gradient(circle, #000000, transparent)"
      .to_string(),
  );

  let container = create_container(background_images);

  run_style_width_test(
    container.into(),
    "tests/fixtures/style_background_image_linear_radial_mixed.png",
  );
}

#[test]
fn test_background_no_repeat_center_with_size_px() {
  let images = BackgroundImagesValue::Css(
    "linear-gradient(90deg, rgba(255,0,0,1), rgba(0,0,255,1))".to_string(),
  );
  let container = create_container_with(
    images,
    Some(BackgroundSizesValue::Css("200px 120px".to_string())),
    Some(BackgroundPositionsValue::Css("center center".to_string())),
    Some(BackgroundRepeatsValue::Css("no-repeat".to_string())),
  );

  run_style_width_test(
    container.into(),
    "tests/fixtures/style_background_no_repeat_center_200x120.png",
  );
}

#[test]
fn test_background_repeat_tile_from_top_left() {
  let images = BackgroundImagesValue::Css(
    "linear-gradient(90deg, rgba(0,200,0,1), rgba(0,0,0,0))".to_string(),
  );
  let container = create_container_with(
    images,
    Some(BackgroundSizesValue::Css("160px 100px".to_string())),
    Some(BackgroundPositionsValue::Css("0 0".to_string())),
    Some(BackgroundRepeatsValue::Css("repeat".to_string())),
  );

  run_style_width_test(
    container.into(),
    "tests/fixtures/style_background_repeat_tile_from_top_left.png",
  );
}

#[test]
fn test_background_repeat_space() {
  let images = BackgroundImagesValue::Css(
    "radial-gradient(circle, rgba(255,165,0,1) 0%, rgba(255,165,0,0) 70%)".to_string(),
  );
  let container = create_container_with(
    images,
    Some(BackgroundSizesValue::Css("120px 120px".to_string())),
    None,
    Some(BackgroundRepeatsValue::Css("space".to_string())),
  );

  run_style_width_test(
    container.into(),
    "tests/fixtures/style_background_repeat_space.png",
  );
}

#[test]
fn test_background_repeat_round() {
  let images = BackgroundImagesValue::Css(
    "radial-gradient(circle, rgba(0,0,0,1) 0%, rgba(0,0,0,0) 60%)".to_string(),
  );
  let container = create_container_with(
    images,
    Some(BackgroundSizesValue::Css("180px 120px".to_string())),
    None,
    Some(BackgroundRepeatsValue::Css("round".to_string())),
  );

  run_style_width_test(
    container.into(),
    "tests/fixtures/style_background_repeat_round.png",
  );
}

#[test]
fn test_background_position_percentage_with_no_repeat() {
  let images = BackgroundImagesValue::Css(
    "linear-gradient(0deg, rgba(255,0,255,1), rgba(255,0,255,0))".to_string(),
  );
  let container = create_container_with(
    images,
    Some(BackgroundSizesValue::Css("220px 160px".to_string())),
    Some(BackgroundPositionsValue::Css("25% 75%".to_string())),
    Some(BackgroundRepeatsValue::Css("no-repeat".to_string())),
  );

  run_style_width_test(
    container.into(),
    "tests/fixtures/style_background_position_percent_25_75.png",
  );
}

#[test]
fn test_background_size_percentage_with_repeat() {
  let images = BackgroundImagesValue::Css(
    "linear-gradient(180deg, rgba(0,128,255,0.9), rgba(0,128,255,0))".to_string(),
  );
  let container = create_container_with(
    images,
    Some(BackgroundSizesValue::Css("20% 20%".to_string())),
    Some(BackgroundPositionsValue::Css("0 0".to_string())),
    Some(BackgroundRepeatsValue::Css("repeat".to_string())),
  );

  run_style_width_test(
    container.into(),
    "tests/fixtures/style_background_size_percent_20_20.png",
  );
}

#[test]
fn test_background_image_grid_pattern() {
  let images = BackgroundImagesValue::Css(
    "linear-gradient(to right, grey 1px, transparent 1px), linear-gradient(to bottom, grey 1px, transparent 1px)".to_string(),
  );

  let mut container = create_container_with(
    images,
    Some(BackgroundSizesValue::Css("40px 40px".to_string())),
    Some(BackgroundPositionsValue::Css("0 0, 0 0".to_string())),
    Some(BackgroundRepeatsValue::Css("repeat, repeat".to_string())),
  );

  container.style.background_color = Color::white().into();

  assert_eq!(
    container.style.background_repeat,
    CssOption::some(BackgroundRepeats(vec![
      BackgroundRepeat::repeat(),
      BackgroundRepeat::repeat()
    ]))
    .into()
  );

  run_style_width_test(
    container.into(),
    "tests/fixtures/style_background_image_grid_pattern.png",
  );
}

#[test]
fn test_background_image_noise_v1_with_gradient() {
  let images = BackgroundImagesValue::Css(
    "radial-gradient(circle at 25% 25%, rgba(255, 0, 128, 0.6), transparent 50%), radial-gradient(circle at 75% 75%, rgba(0, 128, 255, 0.6), transparent 50%), linear-gradient(135deg, rgba(138, 43, 226, 0.4), rgba(30, 144, 255, 0.4), rgba(255, 20, 147, 0.4)), noise-v1(frequency(1.4) octaves(4) opacity(0.8))".to_string(),
  );

  let mut container = create_container_with(
    images,
    Some(BackgroundSizesValue::Css(
      "100% 100%, 100% 100%, 100% 100%, 100% 100%".to_string(),
    )),
    Some(BackgroundPositionsValue::Css(
      "0 0, 0 0, 0 0, 0 0".to_string(),
    )),
    Some(BackgroundRepeatsValue::Css(
      "no-repeat, no-repeat, no-repeat, no-repeat".to_string(),
    )),
  );

  container.style.background_color = Color::white().into();

  run_style_width_test(
    container.into(),
    "tests/fixtures/style_background_image_noise_v1_blend.png",
  );
}
