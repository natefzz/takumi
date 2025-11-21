use cssparser::Parser;

use crate::layout::style::{FromCss, ParseResult, tw::TailwindPropertyParser};

#[derive(Default, Debug, Clone, Copy, PartialEq)]
/// Represents a aspect ratio.
pub enum AspectRatio {
  /// The aspect ratio is determined by the content.
  #[default]
  Auto,
  /// The aspect ratio is a fixed ratio.
  Ratio(f32),
}

impl TailwindPropertyParser for AspectRatio {
  fn parse_tw(token: &str) -> Option<Self> {
    Self::from_str(token).ok()
  }
}

impl From<AspectRatio> for Option<f32> {
  fn from(value: AspectRatio) -> Self {
    match value {
      AspectRatio::Auto => None,
      AspectRatio::Ratio(ratio) => Some(ratio),
    }
  }
}

impl<'i> FromCss<'i> for AspectRatio {
  fn from_css(input: &mut Parser<'i, '_>) -> ParseResult<'i, Self> {
    if input
      .try_parse(|input| input.expect_ident_matching("auto"))
      .is_ok()
    {
      return Ok(AspectRatio::Auto);
    }

    let first_ratio = input.expect_number()?;

    if input.try_parse(|input| input.expect_delim('/')).is_err() {
      return Ok(AspectRatio::Ratio(first_ratio));
    }

    let second_ratio = input.expect_number()?;
    Ok(AspectRatio::Ratio(first_ratio / second_ratio))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use cssparser::{Parser, ParserInput};

  fn parse_aspect_ratio(input: &str) -> ParseResult<'_, AspectRatio> {
    let mut parser_input = ParserInput::new(input);
    let mut parser = Parser::new(&mut parser_input);
    AspectRatio::from_css(&mut parser)
  }

  #[test]
  fn parses_auto_keyword() {
    let result = parse_aspect_ratio("auto").unwrap();
    assert_eq!(result, AspectRatio::Auto);
  }

  #[test]
  fn parses_single_number_as_ratio() {
    let result = parse_aspect_ratio("1.5").unwrap();
    assert_eq!(result, AspectRatio::Ratio(1.5));
  }

  #[test]
  fn parses_ratio_with_slash() {
    let result = parse_aspect_ratio("16/9").unwrap();
    assert_eq!(result, AspectRatio::Ratio(16.0 / 9.0));
  }

  #[test]
  fn parses_ratio_with_decimal_values() {
    let result = parse_aspect_ratio("1.777/1").unwrap();
    assert_eq!(result, AspectRatio::Ratio(1.777));
  }

  #[test]
  fn errors_on_invalid_input() {
    let result = parse_aspect_ratio("invalid");
    assert!(result.is_err());
  }

  #[test]
  fn errors_on_empty_slash() {
    let result = parse_aspect_ratio("16/");
    assert!(result.is_err());
  }
}
