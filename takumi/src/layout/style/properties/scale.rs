use cssparser::{Parser, ParserInput};
use serde::{Deserialize, Serialize};
use taffy::Size;
use ts_rs::TS;

use crate::layout::style::{FromCss, ParseResult, parse_length_percentage};

/// Represents a 2D scale for CSS scale property
#[derive(Debug, Clone, Copy, Deserialize, Serialize, TS, PartialEq)]
#[serde(try_from = "ScaleValue")]
#[ts(as = "ScaleValue")]
pub struct Scale {
  /// Horizontal scaling factor
  pub x: f32,
  /// Vertical scaling factor
  pub y: f32,
}

#[derive(Debug, Clone, PartialEq, TS, Deserialize)]
#[serde(untagged)]
pub(crate) enum ScaleValue {
  Structured { x: f32, y: f32 },
  Css(String),
}

impl TryFrom<ScaleValue> for Scale {
  type Error = String;

  fn try_from(value: ScaleValue) -> Result<Self, Self::Error> {
    match value {
      ScaleValue::Structured { x, y } => Ok(Self { x, y }),
      ScaleValue::Css(css) => {
        let mut input = ParserInput::new(&css);
        let mut parser = Parser::new(&mut input);

        Scale::from_css(&mut parser).map_err(|e| e.to_string())
      }
    }
  }
}

impl From<Scale> for Size<f32> {
  fn from(scale: Scale) -> Self {
    Self {
      width: scale.x,
      height: scale.y,
    }
  }
}

impl Default for Scale {
  fn default() -> Self {
    Self { x: 1.0, y: 1.0 }
  }
}

impl<'i> FromCss<'i> for Scale {
  fn from_css(input: &mut Parser<'i, '_>) -> ParseResult<'i, Self> {
    let first = parse_length_percentage(input)?;
    if let Ok(y) = parse_length_percentage(input) {
      Ok(Self { x: first, y })
    } else {
      Ok(Self { x: first, y: first })
    }
  }
}
