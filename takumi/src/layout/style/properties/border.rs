use cssparser::{Parser, Token, match_ignore_ascii_case};

use crate::layout::style::{ColorInput, FromCss, ParseResult, properties::LengthUnit};

/// Represents border style options (currently only solid is supported).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BorderStyle {
  /// Solid border style.
  Solid,
}

/// Parsed `border` value.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Border {
  /// Border width as a `LengthUnit`.
  pub width: Option<LengthUnit>,
  /// Border style (currently only solid is supported).
  pub style: Option<BorderStyle>,
  /// Optional border color.
  pub color: Option<ColorInput>,
}

impl<'i> FromCss<'i> for Border {
  fn from_css(input: &mut Parser<'i, '_>) -> ParseResult<'i, Self> {
    let mut width = None;
    let mut style = None;
    let mut color = None;

    loop {
      if input.is_exhausted() {
        break;
      }

      if let Ok(value) = input.try_parse(LengthUnit::from_css) {
        width = Some(value);
        continue;
      }

      if let Ok(value) = input.try_parse(BorderStyle::from_css) {
        style = Some(value);
        continue;
      }

      if let Ok(value) = input.try_parse(ColorInput::from_css) {
        color = Some(value);
        continue;
      }

      return Err(input.new_error_for_next_token());
    }

    Ok(Border {
      width,
      style,
      color,
    })
  }
}

impl<'i> FromCss<'i> for BorderStyle {
  fn from_css(input: &mut Parser<'i, '_>) -> ParseResult<'i, Self> {
    let ident = input.expect_ident()?;
    match_ignore_ascii_case! {ident,
      "solid" => Ok(BorderStyle::Solid),
      _ => {
        let token = Token::Ident(ident.clone());
        Err(input.new_basic_unexpected_token_error(token).into())
      },
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::layout::style::Color;

  use super::*;

  #[test]
  fn test_parse_border_style_solid() {
    assert_eq!(BorderStyle::from_str("solid"), Ok(BorderStyle::Solid));
  }

  #[test]
  fn test_parse_border_style_invalid() {
    assert!(BorderStyle::from_str("dashed").is_err());
  }

  #[test]
  fn test_parse_border_width_only() {
    assert_eq!(
      Border::from_str("10px"),
      Ok(Border {
        width: Some(LengthUnit::Px(10.0)),
        style: None,
        color: None,
      })
    );
  }

  #[test]
  fn test_parse_border_style_only() {
    assert_eq!(
      Border::from_str("solid"),
      Ok(Border {
        width: None,
        style: Some(BorderStyle::Solid),
        color: None,
      })
    );
  }

  #[test]
  fn test_parse_border_color_only() {
    assert_eq!(
      Border::from_str("red"),
      Ok(Border {
        width: None,
        style: None,
        color: Some(ColorInput::Value(Color([255, 0, 0, 255]))),
      })
    );
  }

  #[test]
  fn test_parse_border_width_and_style() {
    assert_eq!(
      Border::from_str("2px solid"),
      Ok(Border {
        width: Some(LengthUnit::Px(2.0)),
        style: Some(BorderStyle::Solid),
        color: None,
      })
    );
  }

  #[test]
  fn test_parse_border_width_style_color() {
    assert_eq!(
      Border::from_str("2px solid red"),
      Ok(Border {
        width: Some(LengthUnit::Px(2.0)),
        style: Some(BorderStyle::Solid),
        color: Some(ColorInput::Value(Color([255, 0, 0, 255]))),
      })
    );
  }

  #[test]
  fn test_parse_border_style_width_color() {
    assert_eq!(
      Border::from_str("solid 2px red"),
      Ok(Border {
        width: Some(LengthUnit::Px(2.0)),
        style: Some(BorderStyle::Solid),
        color: Some(ColorInput::Value(Color([255, 0, 0, 255]))),
      })
    );
  }

  #[test]
  fn test_parse_border_color_style_width() {
    assert_eq!(
      Border::from_str("red solid 2px"),
      Ok(Border {
        width: Some(LengthUnit::Px(2.0)),
        style: Some(BorderStyle::Solid),
        color: Some(ColorInput::Value(Color([255, 0, 0, 255]))),
      })
    );
  }

  #[test]
  fn test_parse_border_rem_units() {
    assert_eq!(
      Border::from_str("1.5rem solid blue"),
      Ok(Border {
        width: Some(LengthUnit::Rem(1.5)),
        style: Some(BorderStyle::Solid),
        color: Some(ColorInput::Value(Color([0, 0, 255, 255]))),
      })
    );
  }

  #[test]
  fn test_parse_border_hex_color() {
    assert_eq!(
      Border::from_str("3px solid #ff0000"),
      Ok(Border {
        width: Some(LengthUnit::Px(3.0)),
        style: Some(BorderStyle::Solid),
        color: Some(ColorInput::Value(Color([255, 0, 0, 255]))),
      })
    );
  }

  #[test]
  fn test_parse_border_rgb_color() {
    assert_eq!(
      Border::from_str("4px solid rgb(0, 255, 0)"),
      Ok(Border {
        width: Some(LengthUnit::Px(4.0)),
        style: Some(BorderStyle::Solid),
        color: Some(ColorInput::Value(Color([0, 255, 0, 255]))),
      })
    );
  }

  #[test]
  fn test_parse_border_invalid_style() {
    assert!(Border::from_str("2px dashed red").is_err());
  }

  #[test]
  fn test_parse_border_invalid_color() {
    assert!(Border::from_str("2px solid invalid-color").is_err());
  }

  #[test]
  fn test_parse_border_empty() {
    assert_eq!(Border::from_str(""), Ok(Border::default()));
  }

  #[test]
  fn test_border_value_from_css() {
    assert_eq!(
      Border::from_str("3px solid blue"),
      Ok(Border {
        width: Some(LengthUnit::Px(3.0)),
        style: Some(BorderStyle::Solid),
        color: Some(ColorInput::Value(Color([0, 0, 255, 255]))),
      })
    );
  }

  #[test]
  fn test_border_value_from_invalid_css() {
    assert!(Border::from_str("invalid border").is_err());
  }
}
