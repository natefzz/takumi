use smallvec::smallvec;
use takumi::layout::{
  node::TextNode,
  style::{
    Color, CssOption,
    LengthUnit::{Percentage, Px},
    StyleBuilder, TextAlign, TextDecoration, TextDecorationLine, TextDecorationLines,
  },
};

mod test_utils;
use test_utils::run_style_width_test;

#[test]
fn test_style_text_decoration() {
  let text = TextNode {
    style: StyleBuilder::default()
      .width(Percentage(100.0))
      .text_align(TextAlign::Center)
      .background_color(Color([240, 240, 240, 255]))
      .font_size(CssOption::some(Px(72.0)))
      .text_decoration(TextDecoration {
        line: TextDecorationLines(smallvec![
          TextDecorationLine::Underline,
          TextDecorationLine::LineThrough,
          TextDecorationLine::Overline,
        ]),
        style: None,
        color: Some(Color([255, 0, 0, 255])),
      })
      .build()
      .unwrap(),
    text: "Text Decoration with Underline, Line-Through, and Overline".to_string(),
  };

  run_style_width_test(text.into(), "tests/fixtures/style_text_decoration.png");
}
