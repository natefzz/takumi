use cssparser::{Parser, ParserInput};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::layout::style::{FromCss, LengthUnit, ParseResult};

/// Represents a 2D translation for CSS translate property
#[derive(Debug, Clone, Copy, Deserialize, Serialize, TS, PartialEq)]
#[serde(try_from = "TranslateValue")]
#[ts(as = "TranslateValue")]
pub struct Translate {
  /// Horizontal translation distance
  pub x: LengthUnit,
  /// Vertical translation distance
  pub y: LengthUnit,
}

#[derive(Debug, Clone, PartialEq, TS, Deserialize)]
#[serde(untagged)]
pub(crate) enum TranslateValue {
  Structured { x: LengthUnit, y: LengthUnit },
  Css(String),
}

impl TryFrom<TranslateValue> for Translate {
  type Error = String;

  fn try_from(value: TranslateValue) -> Result<Self, Self::Error> {
    match value {
      TranslateValue::Structured { x, y } => Ok(Self { x, y }),
      TranslateValue::Css(css) => {
        let mut input = ParserInput::new(&css);
        let mut parser = Parser::new(&mut input);

        Translate::from_css(&mut parser).map_err(|e| e.to_string())
      }
    }
  }
}

impl<'i> FromCss<'i> for Translate {
  fn from_css(input: &mut Parser<'i, '_>) -> ParseResult<'i, Self> {
    let first = LengthUnit::from_css(input)?;

    if let Ok(y) = input.try_parse(LengthUnit::from_css) {
      Ok(Self { x: first, y })
    } else {
      Ok(Self { x: first, y: first })
    }
  }
}
