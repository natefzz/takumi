use cssparser::{Parser, ParserInput, match_ignore_ascii_case};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::layout::style::{FromCss, ParseResult};

/// How children overflowing their container should affect layout
#[derive(Debug, Clone, Copy, Deserialize, Serialize, TS, PartialEq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum Overflow {
  /// The automatic minimum size of this node as a flexbox/grid item should be based on the size of its content.
  /// Content that overflows this node *should* contribute to the scroll region of its parent.
  #[default]
  Visible,
  /// The automatic minimum size of this node as a flexbox/grid item should be `0`.
  /// Content that overflows this node should *not* contribute to the scroll region of its parent.
  Hidden,
}

impl From<Overflow> for taffy::Overflow {
  fn from(val: Overflow) -> Self {
    match val {
      Overflow::Visible => taffy::Overflow::Visible,
      Overflow::Hidden => taffy::Overflow::Hidden,
    }
  }
}

impl<'i> FromCss<'i> for Overflow {
  fn from_css(input: &mut Parser<'i, '_>) -> ParseResult<'i, Self> {
    let location = input.current_source_location();
    let ident = input.expect_ident()?;

    match_ignore_ascii_case! { ident,
      "visible" => Ok(Overflow::Visible),
      "hidden" => Ok(Overflow::Hidden),
      _ => Err(location.new_unexpected_token_error(
        cssparser::Token::Ident(ident.clone())
      )),
    }
  }
}

/// Represents overflow values for both axes.
///
/// Can be either a single value applied to both axes, or separate values
/// for horizontal and vertical overflow.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, TS, PartialEq)]
#[serde(try_from = "OverflowValue")]
#[ts(as = "OverflowValue")]
pub struct Overflows(pub Overflow, pub Overflow);

/// Represents a value for the overflow property.
///
/// Can be either a single value applied to both axes, or separate values
/// for horizontal and vertical overflow.
#[derive(Debug, Clone, Deserialize, Serialize, TS, PartialEq)]
#[serde(untagged)]
pub enum OverflowValue {
  /// Same overflow value for both horizontal and vertical
  SingleValue(Overflow),
  /// Separate values for horizontal and vertical overflow (horizontal, vertical)
  Array(Overflow, Overflow),
  /// CSS string representation
  Css(String),
}

impl Default for Overflows {
  fn default() -> Self {
    Self(Overflow::Visible, Overflow::Visible)
  }
}

impl TryFrom<OverflowValue> for Overflows {
  type Error = String;

  fn try_from(value: OverflowValue) -> Result<Self, Self::Error> {
    match value {
      OverflowValue::SingleValue(value) => Ok(Self(value, value)),
      OverflowValue::Array(horizontal, vertical) => Ok(Self(horizontal, vertical)),
      OverflowValue::Css(value) => {
        let mut input = ParserInput::new(&value);
        let mut parser = Parser::new(&mut input);

        let first = Overflow::from_css(&mut parser).map_err(|e| e.to_string())?;

        if let Ok(second) = Overflow::from_css(&mut parser) {
          Ok(Self(first, second))
        } else {
          Ok(Self(first, first))
        }
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_overflow_from_css() {
    let mut input = ParserInput::new("visible");
    let mut parser = Parser::new(&mut input);
    assert_eq!(Overflow::from_css(&mut parser).unwrap(), Overflow::Visible);

    let mut input = ParserInput::new("hidden");
    let mut parser = Parser::new(&mut input);
    assert_eq!(Overflow::from_css(&mut parser).unwrap(), Overflow::Hidden);
  }

  #[test]
  fn test_overflow_from_css_invalid() {
    let mut input = ParserInput::new("invalid");
    let mut parser = Parser::new(&mut input);
    assert!(Overflow::from_css(&mut parser).is_err());
  }

  #[test]
  fn test_overflows_default() {
    assert_eq!(
      Overflows::default(),
      Overflows(Overflow::Visible, Overflow::Visible)
    );
  }

  #[test]
  fn test_overflows_try_from_variants() {
    // TryFrom SingleValue
    let single = OverflowValue::SingleValue(Overflow::Hidden);
    let overflows_single = Overflows::try_from(single).expect("SingleValue should convert");
    assert_eq!(
      overflows_single,
      Overflows(Overflow::Hidden, Overflow::Hidden)
    );

    // TryFrom Array
    let array = OverflowValue::Array(Overflow::Hidden, Overflow::Visible);
    let overflows_array = Overflows::try_from(array).expect("Array should convert");
    assert_eq!(
      overflows_array,
      Overflows(Overflow::Hidden, Overflow::Visible)
    );
  }

  #[test]
  fn test_overflows_from_css_parsing() {
    let overflows_single =
      Overflows::try_from(OverflowValue::Css("hidden".to_string())).expect("hidden parses");
    assert_eq!(
      overflows_single,
      Overflows(Overflow::Hidden, Overflow::Hidden)
    );

    let overflows_two = Overflows::try_from(OverflowValue::Css("hidden visible".to_string()))
      .expect("two values parse");
    assert_eq!(
      overflows_two,
      Overflows(Overflow::Hidden, Overflow::Visible)
    );
  }

  #[test]
  fn test_overflows_from_css_invalid() {
    let res = Overflows::try_from(OverflowValue::Css("invalid".to_string()));
    assert!(res.is_err());
  }
}
