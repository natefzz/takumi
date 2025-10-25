use cssparser::{Parser, ParserInput};
use serde::{Deserialize, Serialize};
use taffy::{LengthPercentage, Size};
use ts_rs::TS;

use crate::{
  layout::style::{FromCss, LengthUnit},
  rendering::RenderContext,
};

/// Represents spacing between flex items.
///
/// Can be either a single value applied to both axes, or separate values
/// for horizontal and vertical spacing.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, TS, PartialEq)]
#[serde(try_from = "GapValue")]
#[ts(as = "GapValue")]
pub struct Gap(pub LengthUnit, pub LengthUnit);

/// Represents a value for the gap property.
///
/// Can be either a single value applied to both axes, or separate values
/// for horizontal and vertical spacing.
#[derive(Debug, Clone, Deserialize, Serialize, TS, PartialEq)]
#[serde(untagged)]
pub(crate) enum GapValue {
  /// Same gap value for both horizontal and vertical spacing
  SingleValue(LengthUnit),
  /// Separate values for horizontal and vertical spacing (horizontal, vertical)
  Array(LengthUnit, LengthUnit),
  /// CSS string representation
  Css(String),
}

impl Default for Gap {
  fn default() -> Self {
    Self(LengthUnit::Px(0.0), LengthUnit::Px(0.0))
  }
}

impl TryFrom<GapValue> for Gap {
  type Error = String;

  fn try_from(value: GapValue) -> Result<Self, Self::Error> {
    match value {
      GapValue::SingleValue(value) => Ok(Self(value, value)),
      GapValue::Array(horizontal, vertical) => Ok(Self(horizontal, vertical)),
      GapValue::Css(value) => {
        let mut input = ParserInput::new(&value);
        let mut parser = Parser::new(&mut input);

        let first = LengthUnit::from_css(&mut parser).map_err(|e| e.to_string())?;

        if let Ok(second) = LengthUnit::from_css(&mut parser) {
          Ok(Self(first, second))
        } else {
          Ok(Self(first, first))
        }
      }
    }
  }
}

impl Gap {
  /// Resolves the gap to a size in length percentages.
  ///
  /// This method converts the gap value to a size in length percentages,
  /// which can be used to set the size of flex items in a flex container.
  pub(crate) fn resolve_to_size(self, context: &RenderContext) -> Size<LengthPercentage> {
    Size {
      height: self.0.resolve_to_length_percentage(context),
      width: self.1.resolve_to_length_percentage(context),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_gap_try_from_variants() {
    // Default value
    assert_eq!(
      Gap::default(),
      Gap(LengthUnit::Px(0.0), LengthUnit::Px(0.0))
    );

    // TryFrom SingleValue
    let single = GapValue::SingleValue(LengthUnit::Px(12.0));
    let gap_single = Gap::try_from(single).expect("SingleValue should convert");
    assert_eq!(gap_single, Gap(LengthUnit::Px(12.0), LengthUnit::Px(12.0)));

    // TryFrom Array
    let array = GapValue::Array(LengthUnit::Px(5.0), LengthUnit::Px(7.0));
    let gap_array = Gap::try_from(array).expect("Array should convert");
    assert_eq!(gap_array, Gap(LengthUnit::Px(5.0), LengthUnit::Px(7.0)));
  }

  #[test]
  fn test_gap_from_css_parsing() {
    let gap_single = Gap::try_from(GapValue::Css("10px".to_string())).expect("10px parses");
    assert_eq!(gap_single, Gap(LengthUnit::Px(10.0), LengthUnit::Px(10.0)));

    let gap_two = Gap::try_from(GapValue::Css("10px 20px".to_string())).expect("two values parse");
    assert_eq!(gap_two, Gap(LengthUnit::Px(10.0), LengthUnit::Px(20.0)));
  }

  #[test]
  fn test_gap_from_css_invalid() {
    let res = Gap::try_from(GapValue::Css("invalid".to_string()));
    assert!(res.is_err());
  }
}
