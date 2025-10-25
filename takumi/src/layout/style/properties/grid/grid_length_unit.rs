use cssparser::{Parser, ParserInput, Token};
use serde::{Deserialize, Serialize};
use taffy::CompactLength;
use ts_rs::TS;

use crate::{
  layout::style::{FromCss, LengthUnit, ParseResult},
  rendering::RenderContext,
};

/// Represents a fraction of the available space
#[derive(Debug, Clone, Deserialize, Serialize, TS, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum FrLengthUnit {
  /// A fraction of the available space
  Fr(f32),
}

/// Represents a grid track sizing function with serde support
#[derive(Debug, Clone, Deserialize, Serialize, TS, PartialEq)]
#[serde(try_from = "GridLengthUnitValue")]
#[ts(as = "GridLengthUnitValue")]
pub enum GridLengthUnit {
  /// A fraction of the available space
  Fr(f32),
  /// A fixed length
  Unit(LengthUnit),
}

/// Represents a grid length unit value with serde support
#[derive(Debug, Clone, Deserialize, Serialize, TS, PartialEq)]
#[serde(untagged)]
pub(crate) enum GridLengthUnitValue {
  /// A fraction of the available space
  Fr(FrLengthUnit),
  /// A fixed length
  Unit(LengthUnit),
  /// A CSS string representation
  Css(String),
}

impl TryFrom<GridLengthUnitValue> for GridLengthUnit {
  type Error = String;

  fn try_from(value: GridLengthUnitValue) -> Result<Self, Self::Error> {
    match value {
      GridLengthUnitValue::Fr(FrLengthUnit::Fr(fr)) => Ok(GridLengthUnit::Fr(fr)),
      GridLengthUnitValue::Unit(unit) => Ok(GridLengthUnit::Unit(unit)),
      GridLengthUnitValue::Css(css) => {
        let mut input = ParserInput::new(&css);
        let mut parser = Parser::new(&mut input);
        GridLengthUnit::from_css(&mut parser).map_err(|e| e.to_string())
      }
    }
  }
}

impl GridLengthUnit {
  /// Converts the grid track size to a compact length representation.
  pub fn to_compact_length(&self, context: &RenderContext) -> CompactLength {
    match self {
      GridLengthUnit::Fr(fr) => CompactLength::fr(*fr),
      GridLengthUnit::Unit(unit) => unit.to_compact_length(context),
    }
  }
}

// Minimal CSS parsing helpers for grid values (mirror patterns used in other property modules)
impl<'i> FromCss<'i> for GridLengthUnit {
  fn from_css(input: &mut Parser<'i, '_>) -> ParseResult<'i, Self> {
    if let Ok(unit) = input.try_parse(LengthUnit::from_css) {
      return Ok(GridLengthUnit::Unit(unit));
    }

    let location = input.current_source_location();
    let token = input.next()?;

    let Token::Dimension { value, unit, .. } = &token else {
      return Err(
        location
          .new_basic_unexpected_token_error(token.clone())
          .into(),
      );
    };

    if !unit.eq_ignore_ascii_case("fr") {
      return Err(
        location
          .new_basic_unexpected_token_error(token.clone())
          .into(),
      );
    }

    Ok(GridLengthUnit::Fr(*value))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_parse_fr_and_unit() {
    let mut parser_input = ParserInput::new("1fr");
    let mut parser = Parser::new(&mut parser_input);
    let fr = GridLengthUnit::from_css(&mut parser).unwrap();
    assert_eq!(fr, GridLengthUnit::Fr(1.0));

    let mut parser_input = ParserInput::new("10px");
    let mut parser = Parser::new(&mut parser_input);
    let px = GridLengthUnit::from_css(&mut parser).unwrap();
    assert_eq!(px, GridLengthUnit::Unit(LengthUnit::Px(10.0)));
  }
}
