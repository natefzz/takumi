use std::fmt::Display;

use csscolorparser::{NAMED_COLORS, ParseColorError};
use cssparser::{Parser, ParserInput, ToCss, Token};
use image::Rgba;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::layout::style::{FromCss, ParseResult};

/// `Color` proxy type for deserializing CSS color values.
#[derive(Debug, Clone, Deserialize, TS)]
#[serde(untagged)]
pub(crate) enum ColorInputValue {
  /// RGB color with 8-bit components
  Rgb(u8, u8, u8),
  /// RGBA color with 8-bit RGB components and 32-bit float alpha (alpha is between 0.0 and 1.0)
  Rgba(u8, u8, u8, f32),
  /// Single 32-bit integer containing RGB values
  RgbInt(u32),
  /// CSS color string
  #[ts(type = "\"currentColor\" | string")]
  Css(String),
}

/// Represents a color with 8-bit RGBA components.
#[derive(Debug, Clone, PartialEq, Serialize, TS, Copy)]
pub struct Color(pub [u8; 4]);

/// Represents a color input value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS, Copy)]
#[serde(try_from = "ColorInputValue")]
#[ts(as = "ColorInputValue")]
pub enum ColorInput {
  #[serde(rename = "currentColor")]
  /// Inherit from the `color` value.
  CurrentColor,
  /// A color value.
  #[serde(untagged)]
  Value(Color),
}

impl ColorInput {
  /// Resolves the color input to a color.
  pub fn resolve(self, current_color: Color, opacity: f32) -> Color {
    match self {
      ColorInput::Value(color) => color.with_opacity(opacity),
      ColorInput::CurrentColor => current_color.with_opacity(opacity),
    }
  }
}

impl From<Color> for ColorInput {
  fn from(color: Color) -> Self {
    ColorInput::Value(color)
  }
}

impl From<Color> for Rgba<u8> {
  fn from(color: Color) -> Self {
    Rgba(color.0)
  }
}

impl Display for Color {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "rgb({} {} {} / {})",
      self.0[0],
      self.0[1],
      self.0[2],
      self.0[3] as f32 / 255.0
    )
  }
}

impl Default for Color {
  fn default() -> Self {
    Self::transparent()
  }
}

impl Color {
  /// Creates a new transparent color.
  pub const fn transparent() -> Self {
    Color([0, 0, 0, 0])
  }

  /// Creates a new black color.
  pub const fn black() -> Self {
    Color([0, 0, 0, 255])
  }

  /// Creates a new white color.
  pub const fn white() -> Self {
    Color([255, 255, 255, 255])
  }

  /// Apply opacity to alpha channel
  pub fn with_opacity(mut self, opacity: f32) -> Self {
    self.0[3] = ((self.0[3] as f32) * opacity).round() as u8;

    self
  }
}

impl TryFrom<ColorInputValue> for ColorInput {
  type Error = String;

  fn try_from(value: ColorInputValue) -> Result<Self, Self::Error> {
    match value {
      ColorInputValue::Rgb(r, g, b) => Ok(ColorInput::Value(Color([r, g, b, 255]))),
      ColorInputValue::Rgba(r, g, b, a) => {
        Ok(ColorInput::Value(Color([r, g, b, (a * 255.0) as u8])))
      }
      ColorInputValue::RgbInt(rgb) => {
        let r = ((rgb >> 16) & 0xFF) as u8;
        let g = ((rgb >> 8) & 0xFF) as u8;
        let b = (rgb & 0xFF) as u8;

        Ok(ColorInput::Value(Color([r, g, b, 255])))
      }
      ColorInputValue::Css(css) => {
        let mut input = ParserInput::new(&css);
        let mut parser = Parser::new(&mut input);

        ColorInput::from_css(&mut parser).map_err(|e| e.to_string())
      }
    }
  }
}

impl<'i> FromCss<'i> for ColorInput {
  fn from_css(input: &mut Parser<'i, '_>) -> ParseResult<'i, Self> {
    let location = input.current_source_location();
    let position = input.position();
    let token = input.next()?;

    match *token {
      Token::Hash(_) | Token::IDHash(_) => parse_color_string(&token.to_css_string())
        .map(ColorInput::Value)
        .map_err(|_| {
          location
            .new_basic_unexpected_token_error(token.clone())
            .into()
        }),
      Token::Ident(ref ident) => {
        if ident.eq_ignore_ascii_case("transparent") {
          return Ok(ColorInput::Value(Color([0, 0, 0, 0])));
        }

        if ident.eq_ignore_ascii_case("currentcolor") {
          return Ok(ColorInput::CurrentColor);
        }

        let Some([r, g, b]) = NAMED_COLORS.get(ident) else {
          return Err(
            location
              .new_basic_unexpected_token_error(token.clone())
              .into(),
          );
        };

        Ok(ColorInput::Value(Color([*r, *g, *b, 255])))
      }
      Token::Function(_) => {
        // Have to clone to persist token, and allow input to be borrowed
        let token = token.clone();

        input.parse_nested_block(|input| {
          while input.next().is_ok() {}

          // Slice from the function name till before the closing parenthesis
          let body = input.slice_from(position);

          let mut function = body.to_string();

          // Add closing parenthesis
          function.push(')');

          parse_color_string(&function)
            .map(ColorInput::Value)
            .map_err(|_| location.new_basic_unexpected_token_error(token).into())
        })
      }
      _ => Err(
        location
          .new_basic_unexpected_token_error(token.clone())
          .into(),
      ),
    }
  }
}

fn parse_color_string(string: &str) -> Result<Color, ParseColorError> {
  csscolorparser::parse(string).map(|color| Color(color.to_rgba8()))
}

#[cfg(test)]
mod tests {
  use super::*;
  use cssparser::{Parser, ParserInput};

  fn parse_color_str(input: &str) -> ParseResult<'_, ColorInput> {
    let mut parser_input = ParserInput::new(input);
    let mut parser = Parser::new(&mut parser_input);

    ColorInput::from_css(&mut parser)
  }

  #[test]
  fn test_parse_hex_color_3_digits() {
    // Test 3-digit hex color
    let result = parse_color_str("#f09").unwrap();
    assert_eq!(result, ColorInput::Value(Color([255, 0, 153, 255])));
  }

  #[test]
  fn test_parse_hex_color_6_digits() {
    // Test 6-digit hex color
    let result = parse_color_str("#ff0099").unwrap();
    assert_eq!(result, ColorInput::Value(Color([255, 0, 153, 255])));
  }

  #[test]
  fn test_parse_color_transparent() {
    // Test parsing transparent keyword
    let result = parse_color_str("transparent").unwrap();
    assert_eq!(result, ColorInput::Value(Color([0, 0, 0, 0])));
  }

  #[test]
  fn test_parse_color_rgb_function() {
    // Test parsing rgb() function through main parse function
    let result = parse_color_str("rgb(255, 0, 153)").unwrap();
    assert_eq!(result, ColorInput::Value(Color([255, 0, 153, 255])));
  }

  #[test]
  fn test_parse_color_rgba_function() {
    // Test parsing rgba() function through main parse function
    let result = parse_color_str("rgba(255, 0, 153, 0.5)").unwrap();
    assert_eq!(result, ColorInput::Value(Color([255, 0, 153, 128])));
  }

  #[test]
  fn test_parse_color_rgb_space_separated() {
    // Test parsing rgb() function with space-separated values
    let result = parse_color_str("rgb(255 0 153)").unwrap();
    assert_eq!(result, ColorInput::Value(Color([255, 0, 153, 255])));
  }

  #[test]
  fn test_parse_color_rgb_with_alpha_slash() {
    // Test parsing rgb() function with alpha value using slash
    let result = parse_color_str("rgb(255 0 153 / 0.5)").unwrap();
    assert_eq!(result, ColorInput::Value(Color([255, 0, 153, 128])));
  }

  #[test]
  fn test_parse_named_color_grey() {
    let result = parse_color_str("grey").unwrap();
    assert_eq!(result, ColorInput::Value(Color([128, 128, 128, 255])));
  }

  #[test]
  fn test_parse_color_invalid_function() {
    // Test parsing invalid function
    let result = parse_color_str("invalid(255, 0, 153)");
    assert!(result.is_err());
  }
}
