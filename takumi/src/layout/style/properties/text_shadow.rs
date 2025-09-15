use std::{borrow::Cow, fmt::Debug};

use cssparser::{BasicParseErrorKind, ParseError, Parser, ParserInput};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use ts_rs::TS;

use crate::layout::style::{Color, FromCss, LengthUnit, ParseResult};

/// Represents a text shadow with all its properties.
#[derive(Debug, Clone, PartialEq, Copy, Serialize, Deserialize, TS)]
#[ts(as = "TextShadowValue")]
#[serde(try_from = "TextShadowValue")]
pub struct TextShadow {
  /// Horizontal offset of the shadow.
  pub offset_x: LengthUnit,
  /// Vertical offset of the shadow.
  pub offset_y: LengthUnit,
  /// Blur radius of the shadow. Higher values create a more blurred shadow.
  pub blur_radius: LengthUnit,
  /// Color of the shadow.
  pub color: Color,
}

/// Proxy type for `TextShadow` Css deserialization.
#[derive(Debug, Clone, PartialEq, TS, Deserialize)]
#[serde(untagged)]
pub(crate) enum TextShadowValue {
  /// Represents a structured box shadow.
  #[serde(rename_all = "camelCase")]
  Structured {
    /// Horizontal offset of the shadow.
    offset_x: LengthUnit,
    /// Vertical offset of the shadow.
    offset_y: LengthUnit,
    /// Blur radius of the shadow. Higher values create a more blurred shadow.
    blur_radius: LengthUnit,
    /// Color of the shadow.
    color: Color,
  },
  /// Represents a CSS string.
  Css(String),
}

impl TryFrom<TextShadowValue> for TextShadow {
  type Error = String;

  fn try_from(value: TextShadowValue) -> Result<Self, Self::Error> {
    match value {
      TextShadowValue::Structured {
        offset_x,
        offset_y,
        blur_radius,
        color,
      } => Ok(TextShadow {
        offset_x,
        offset_y,
        blur_radius,
        color,
      }),
      TextShadowValue::Css(css) => {
        let mut input = ParserInput::new(&css);
        let mut parser = Parser::new(&mut input);

        TextShadow::from_css(&mut parser).map_err(|e| e.to_string())
      }
    }
  }
}

/// Represents a collection of text shadows; has custom `FromCss` implementation for comma-separated values.
#[derive(Debug, Clone, PartialEq, Deserialize, TS, Serialize)]
#[ts(as = "TextShadowsValue")]
#[serde(try_from = "TextShadowsValue")]
pub struct TextShadows(pub SmallVec<[TextShadow; 4]>);

#[derive(Debug, Clone, PartialEq, TS, Deserialize)]
#[serde(untagged)]
pub(crate) enum TextShadowsValue {
  #[ts(as = "Vec<TextShadow>")]
  Structured(SmallVec<[TextShadow; 4]>),
  Css(String),
}

impl TryFrom<TextShadowsValue> for TextShadows {
  type Error = String;

  fn try_from(value: TextShadowsValue) -> Result<Self, Self::Error> {
    match value {
      TextShadowsValue::Structured(shadows) => Ok(TextShadows(shadows)),
      TextShadowsValue::Css(css) => {
        let mut input = ParserInput::new(&css);
        let mut parser = Parser::new(&mut input);

        let mut shadows = SmallVec::new();

        loop {
          let Ok(shadow) = TextShadow::from_css(&mut parser) else {
            break;
          };

          shadows.push(shadow);

          if parser.expect_comma().is_err() {
            break;
          }
        }

        Ok(TextShadows(shadows))
      }
    }
  }
}

impl<'i> FromCss<'i> for TextShadow {
  /// Parses a text-shadow value from CSS input.
  ///
  /// The text-shadow syntax supports the following components (in that order):
  /// - Two length values for horizontal and vertical offsets (required)
  /// - An optional length value for blur radius
  /// - An optional color value
  ///
  /// Examples:
  /// - `text-shadow: 2px 4px;`
  /// - `text-shadow: 2px 4px 6px;`
  /// - `text-shadow: 2px 4px red;`
  fn from_css(input: &mut Parser<'i, '_>) -> ParseResult<'i, TextShadow> {
    let mut color = None;
    let mut lengths = None;

    // Parse all components in a loop, as they can appear in any order
    loop {
      // Try to parse length values (offsets, blur radius, spread radius)
      if lengths.is_none() {
        let value = input.try_parse::<_, _, ParseError<Cow<'i, str>>>(|input| {
          // Parse the required horizontal and vertical offsets
          let horizontal = LengthUnit::from_css(input)?;
          let vertical = LengthUnit::from_css(input)?;

          // Parse optional blur radius (defaults to 0)
          let blur = input
            .try_parse(LengthUnit::from_css)
            .unwrap_or(LengthUnit::zero());

          Ok((horizontal, vertical, blur))
        });

        if let Ok(value) = value {
          lengths = Some(value);
          continue;
        }
      }

      // Try to parse a color value if not already found
      if color.is_none() {
        if let Ok(value) = input.try_parse(Color::from_css) {
          color = Some(value);
          continue;
        }
      }

      // If we can't parse anything else, break out of the loop
      break;
    }

    // At minimum, we need the two required length values (offsets)
    let lengths = lengths.ok_or(input.new_error(BasicParseErrorKind::QualifiedRuleInvalid))?;

    // Construct the TextShadow with parsed values or defaults
    Ok(TextShadow {
      // Use parsed color or default to transparent
      color: color.unwrap_or_else(Color::transparent),
      offset_x: lengths.0,
      offset_y: lengths.1,
      blur_radius: lengths.2,
    })
  }
}

#[cfg(test)]
mod tests {
  use crate::layout::style::LengthUnit::Px;

  use super::*;

  #[test]
  fn test_parse_text_shadow_no_blur_radius() {
    let result: TextShadows = TextShadowsValue::Css("5px 5px #558abb".to_string())
      .try_into()
      .unwrap();

    assert_eq!(result.0.len(), 1);

    let shadow = result.0[0];

    assert_eq!(shadow.offset_x, Px(5.0));
    assert_eq!(shadow.offset_y, Px(5.0));
    assert_eq!(shadow.blur_radius, Px(0.0));
    assert_eq!(shadow.color, Color([85, 138, 187, 255]));
  }

  #[test]
  fn test_parse_text_shadow_multiple_values() {
    let result: TextShadows =
      TextShadowsValue::Css("5px 5px #558abb, 10px 10px #558abb".to_string())
        .try_into()
        .unwrap();

    assert_eq!(result.0.len(), 2);
  }
}
