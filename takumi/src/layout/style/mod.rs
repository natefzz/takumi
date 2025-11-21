mod properties;
mod stylesheets;

/// Handle Tailwind CSS properties.
pub mod tw;

use cssparser::match_ignore_ascii_case;
pub use properties::*;
use serde::{Deserialize, Deserializer, de::Error as DeError};
use serde_untagged::UntaggedEnumVisitor;
pub use stylesheets::*;

/// Represents a CSS property value that can be explicitly set, inherited from parent, or reset to initial value.
#[derive(Clone, Debug, PartialEq)]
pub enum CssValue<T, const DEFAULT_INHERIT: bool = false> {
  /// Use the initial value of the property
  Initial,
  /// Inherit the computed value from the parent element
  Inherit,
  /// Explicit value set on the element
  Value(T),
}

impl<'de, T: for<'i> FromCss<'i>, const DEFAULT_INHERIT: bool> Deserialize<'de>
  for CssValue<T, DEFAULT_INHERIT>
{
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    UntaggedEnumVisitor::new()
      .expecting("`initial` or `inherit` or T")
      .string(|str| {
        match_ignore_ascii_case! {str,
          "initial" => Ok(CssValue::Initial),
          "inherit" => Ok(CssValue::Inherit),
          _ => T::from_str(str).map(CssValue::Value).map_err(DeError::custom),
        }
      })
      .i64(|num| {
        T::from_str(num.to_string().as_str())
          .map(CssValue::Value)
          .map_err(DeError::custom)
      })
      .f64(|num| {
        T::from_str(num.to_string().as_str())
          .map(CssValue::Value)
          .map_err(DeError::custom)
      })
      .deserialize(deserializer)
  }
}

impl<'de, T: for<'i> FromCss<'i>, const DEFAULT_INHERIT: bool> Deserialize<'de>
  for CssValue<Option<T>, DEFAULT_INHERIT>
{
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    UntaggedEnumVisitor::new()
      .expecting("`initial` or `inherit` or T")
      .string(|str| {
        match_ignore_ascii_case! {str,
          "none" => Ok(CssValue::Value(None)),
          "initial" => Ok(CssValue::Initial),
          "inherit" => Ok(CssValue::Inherit),
          _ => T::from_str(str).map(|value| CssValue::Value(Some(value))).map_err(DeError::custom)
        }
      })
      .i64(|num| {
        T::from_str(num.to_string().as_str())
          .map(|value| CssValue::Value(Some(value)))
          .map_err(DeError::custom)
      })
      .f64(|num| {
        T::from_str(num.to_string().as_str())
          .map(|value| CssValue::Value(Some(value)))
          .map_err(DeError::custom)
      })
      .deserialize(deserializer)
  }
}

impl<T: Default, const DEFAULT_INHERIT: bool> Default for CssValue<T, DEFAULT_INHERIT> {
  fn default() -> Self {
    if DEFAULT_INHERIT {
      CssValue::Inherit
    } else {
      CssValue::Initial
    }
  }
}

impl<T, const DEFAULT_INHERIT: bool> From<T> for CssValue<T, DEFAULT_INHERIT> {
  fn from(value: T) -> Self {
    CssValue::Value(value)
  }
}

impl<T: Default, const DEFAULT_INHERIT: bool> CssValue<T, DEFAULT_INHERIT> {
  /// Resolves this CssValue to a concrete value based on inheritance rules
  pub(crate) fn inherit_value(self, parent: &T) -> T
  where
    T: Clone,
  {
    match self {
      Self::Value(v) => v,
      Self::Inherit => parent.clone(),
      Self::Initial => T::default(),
    }
  }
}

impl<T: Copy> Copy for CssValue<T> {}
