use cssparser::Parser;
use taffy::Rect;

use crate::layout::style::{FromCss, LengthUnit, ParseResult};

/// Represents the values for the four sides of a box (top, right, bottom, left).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sides<T: Copy>(pub [T; 4]);

pub(crate) enum Axis {
  Horizontal,
  Vertical,
}

impl<T: Copy> Sides<T> {
  pub(crate) fn map_axis<R: Copy, F: Fn(T, Axis) -> R>(&self, func: F) -> Sides<R> {
    let [top, right, bottom, left] = self.0;

    Sides([
      func(top, Axis::Vertical),
      func(right, Axis::Horizontal),
      func(bottom, Axis::Vertical),
      func(left, Axis::Horizontal),
    ])
  }
}

impl<'i, T: Copy + for<'j> FromCss<'j>> FromCss<'i> for Sides<T> {
  fn from_css(input: &mut Parser<'i, '_>) -> ParseResult<'i, Self> {
    // Parse between 1 and 4 values of T using FromCss
    let first = T::from_css(input)?;

    // Collect all values by parsing until we can't parse more
    let mut values = Vec::with_capacity(4);

    values.push(first);

    // Keep parsing values separated by whitespace
    loop {
      // Try to parse the next value
      match input.try_parse(T::from_css) {
        Ok(next_value) => values.push(next_value),
        Err(_) => break,
      }

      // Don't allow more than 4 values
      if values.len() >= 4 {
        break;
      }
    }

    // Now create the sides based on how many values we got
    let sides = match values.len() {
      1 => Sides([values[0]; 4]),
      2 => Sides([values[0], values[1], values[0], values[1]]),
      3 => Sides([values[0], values[1], values[2], values[1]]),
      4 => Sides([values[0], values[1], values[2], values[3]]),
      _ => unreachable!(),
    };

    Ok(sides)
  }
}

impl<T: Copy> From<Sides<T>> for Rect<T> {
  fn from(value: Sides<T>) -> Self {
    Rect {
      top: value.0[0],
      right: value.0[1],
      bottom: value.0[2],
      left: value.0[3],
    }
  }
}

impl<T: Default + Copy> Default for Sides<T> {
  fn default() -> Self {
    Self([T::default(); 4])
  }
}

impl<T: Copy> From<T> for Sides<T> {
  fn from(value: T) -> Self {
    Self([value; 4])
  }
}

impl Sides<LengthUnit> {
  /// Creates a new zeroable Sides with [LengthUnit::zero].
  pub const fn zero() -> Self {
    Self([LengthUnit::zero(); 4])
  }

  /// Creates a new autoable Sides with [LengthUnit::Auto].
  pub const fn auto() -> Self {
    Self([LengthUnit::Auto; 4])
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::layout::style::LengthUnit;

  #[test]
  fn deserialize_single_number() {
    let json = "5";
    let sides: Sides<LengthUnit> = Sides::from_str(json).expect("should deserialize");
    assert_eq!(sides, Sides([LengthUnit::Px(5.0); 4]));
  }

  #[test]
  fn deserialize_axis_pair_numbers() {
    let sides: Sides<LengthUnit> = Sides::from_str("10 20").unwrap();

    assert_eq!(
      sides,
      Sides([
        LengthUnit::Px(10.0),
        LengthUnit::Px(20.0),
        LengthUnit::Px(10.0),
        LengthUnit::Px(20.0)
      ])
    );
  }

  #[test]
  fn deserialize_css_single_value() {
    let sides: Sides<LengthUnit> = Sides::from_str("10px").unwrap();

    assert_eq!(sides, Sides([LengthUnit::Px(10.0); 4]));
  }

  #[test]
  fn deserialize_css_multi_values() {
    let sides: Sides<LengthUnit> = Sides::from_str("1px 2px 3px 4px").unwrap();

    assert_eq!(
      sides,
      Sides([
        LengthUnit::Px(1.0),
        LengthUnit::Px(2.0),
        LengthUnit::Px(3.0),
        LengthUnit::Px(4.0)
      ])
    );
  }
}
