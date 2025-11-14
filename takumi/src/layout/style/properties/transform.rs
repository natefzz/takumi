use std::ops::{Deref, DerefMut};

use cssparser::{Parser, Token, match_ignore_ascii_case};
use serde::{Deserialize, Deserializer, Serialize, Serializer, ser::SerializeSeq};
use smallvec::SmallVec;
use taffy::{Point, Size};
use ts_rs::TS;

use crate::{
  layout::style::{Angle, FromCss, LengthUnit, ParseResult, PercentageNumber},
  rendering::RenderContext,
};

const DEFAULT_SCALE: f32 = 1.0;

/// Represents a single CSS transform operation
#[derive(Debug, Clone, Deserialize, Serialize, Copy, TS, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Transform {
  /// Translates an element along the X-axis and Y-axis by the specified lengths
  Translate(LengthUnit, LengthUnit),
  /// Scales an element by the specified factors
  Scale(f32, f32),
  /// Rotates an element (2D rotation) by angle in degrees
  Rotate(Angle),
  /// Skews an element by the specified angles
  Skew(Angle, Angle),
  /// Applies raw affine matrix values
  Matrix(Affine),
}

/// | a c x |
/// | b d y |
/// | 0 0 1 |
#[derive(Debug, Clone, Copy, TS, Default)]
#[ts(type = "string")]
pub struct Affine(zeno::Transform);

impl Affine {
  /// Returns the identity transform
  pub const fn identity() -> Self {
    Self(zeno::Transform::IDENTITY)
  }

  /// Returns true if the transform is the identity transform
  pub fn is_identity(self) -> bool {
    self == Self::identity()
  }

  /// Decomposes the translation part of the transform
  pub fn decompose_translation(self) -> Point<f32> {
    Point {
      x: self.x,
      y: self.y,
    }
  }

  /// Decomposes the scale part of the transform
  pub fn decompose_scale(self) -> Size<f32> {
    Size {
      width: (self.xx * self.xx + self.xy * self.xy).sqrt(),
      height: (self.yx * self.yx + self.yy * self.yy).sqrt(),
    }
  }

  /// Returns true if the transform is only a translation
  pub(crate) fn only_translation(self) -> bool {
    self.xx == Self::identity().xx
      && self.xy == Self::identity().xy
      && self.yx == Self::identity().yx
      && self.yy == Self::identity().yy
  }
}

impl Deref for Affine {
  type Target = zeno::Transform;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for Affine {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl PartialEq for Affine {
  fn eq(&self, other: &Self) -> bool {
    self.xx == other.xx
      && self.xy == other.xy
      && self.yx == other.yx
      && self.yy == other.yy
      && self.x == other.x
      && self.y == other.y
  }
}

impl<'de> Deserialize<'de> for Affine {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let seq = String::deserialize(deserializer)?;
    Affine::from_str(&seq).map_err(|e| serde::de::Error::custom(e.to_string()))
  }
}

impl Serialize for Affine {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    let mut seq = serializer.serialize_seq(Some(6))?;
    seq.serialize_element(&self.0.xx)?;
    seq.serialize_element(&self.0.xy)?;
    seq.serialize_element(&self.0.yx)?;
    seq.serialize_element(&self.0.yy)?;
    seq.serialize_element(&self.0.x)?;
    seq.serialize_element(&self.0.y)?;
    seq.end()
  }
}

impl<'i> FromCss<'i> for Affine {
  fn from_css(input: &mut Parser<'i, '_>) -> ParseResult<'i, Self> {
    let a = input.expect_number()?;
    let b = input.expect_number()?;
    let c = input.expect_number()?;
    let d = input.expect_number()?;
    let x = input.expect_number()?;
    let y = input.expect_number()?;

    Ok(Affine(zeno::Transform::new(a, b, c, d, x, y)))
  }
}

impl From<zeno::Transform> for Affine {
  fn from(transform: zeno::Transform) -> Self {
    Affine(transform)
  }
}

/// A collection of transform operations that can be applied together
#[derive(Debug, Clone, Deserialize, Serialize, TS, Default, PartialEq)]
#[ts(as = "TransformsValue")]
#[serde(try_from = "TransformsValue")]
pub struct Transforms(pub SmallVec<[Transform; 4]>);

impl Transforms {
  /// Converts the transforms to a [`Affine`] instance
  pub(crate) fn to_affine(&self, context: &RenderContext, border_box: Size<f32>) -> Affine {
    let mut instance = zeno::Transform::IDENTITY;

    for transform in self.0.iter() {
      match *transform {
        Transform::Translate(x_length, y_length) => {
          instance = instance.then_translate(
            x_length.resolve_to_px(context, border_box.width),
            y_length.resolve_to_px(context, border_box.height),
          );
        }
        Transform::Scale(x_scale, y_scale) => {
          instance = instance.then_scale(x_scale, y_scale);
        }
        Transform::Rotate(angle) => {
          instance = instance.then_rotate(angle.into());
        }
        Transform::Skew(x_angle, y_angle) => {
          instance = instance.then(&zeno::Transform::skew(x_angle.into(), y_angle.into()));
        }
        Transform::Matrix(affine) => {
          instance = instance.then(&affine);
        }
      }
    }

    instance.into()
  }
}

/// Represents transform values that can be either a structured list or raw CSS
#[derive(Debug, Clone, Deserialize, TS)]
#[serde(untagged)]
pub(crate) enum TransformsValue {
  /// A structured list of transform operations
  #[ts(as = "Vec<Transform>")]
  Transforms(SmallVec<[Transform; 4]>),
  /// Raw CSS transform string to be parsed
  Css(String),
}

impl<'i> FromCss<'i> for Transforms {
  fn from_css(input: &mut Parser<'i, '_>) -> ParseResult<'i, Self> {
    let mut transforms = SmallVec::new();

    while !input.is_exhausted() {
      let transform = Transform::from_css(input)?;
      transforms.push(transform);
    }

    Ok(Transforms(transforms))
  }
}

impl TryFrom<TransformsValue> for Transforms {
  type Error = String;

  fn try_from(value: TransformsValue) -> Result<Self, Self::Error> {
    match value {
      TransformsValue::Transforms(transforms) => Ok(Transforms(transforms)),
      TransformsValue::Css(css) => Transforms::from_str(&css).map_err(|e| e.to_string()),
    }
  }
}

impl<'i> FromCss<'i> for Transform {
  fn from_css(parser: &mut Parser<'i, '_>) -> ParseResult<'i, Self> {
    let location = parser.current_source_location();
    let token = parser.next()?;

    let Token::Function(function) = token else {
      return Err(
        location
          .new_basic_unexpected_token_error(token.clone())
          .into(),
      );
    };

    match_ignore_ascii_case! {function,
      "translate" => parser.parse_nested_block(|input| {
        let x = LengthUnit::from_css(input)?;
        input.expect_comma()?;
        let y = LengthUnit::from_css(input)?;

        Ok(Transform::Translate(x, y))
      }),
      "translatex" => parser.parse_nested_block(|input| Ok(Transform::Translate(
        LengthUnit::from_css(input)?,
        LengthUnit::zero(),
      ))),
      "translatey" => parser.parse_nested_block(|input| Ok(Transform::Translate(
        LengthUnit::zero(),
        LengthUnit::from_css(input)?,
      ))),
      "scale" => parser.parse_nested_block(|input| {
        let PercentageNumber(x) = PercentageNumber::from_css(input)?;
        if input.try_parse(Parser::expect_comma).is_ok() {
          let PercentageNumber(y) = PercentageNumber::from_css(input)?;
          Ok(Transform::Scale(x, y))
        } else {
          Ok(Transform::Scale(x, x))
        }
      }),
      "scalex" => parser.parse_nested_block(|input| Ok(Transform::Scale(
        PercentageNumber::from_css(input)?.0,
        DEFAULT_SCALE,
      ))),
      "scaley" => parser.parse_nested_block(|input| Ok(Transform::Scale(
        DEFAULT_SCALE,
        PercentageNumber::from_css(input)?.0,
      ))),
      "skew" => parser.parse_nested_block(|input| {
        let x = Angle::from_css(input)?;
        input.expect_comma()?;
        let y = Angle::from_css(input)?;

        Ok(Transform::Skew(x, y))
      }),
      "skewx" => parser.parse_nested_block(|input| Ok(Transform::Skew(
        Angle::from_css(input)?,
        Angle::default(),
      ))),
      "skewy" => parser.parse_nested_block(|input| Ok(Transform::Skew(
        Angle::default(),
        Angle::from_css(input)?,
      ))),
      "rotate" => parser.parse_nested_block(|input| Ok(Transform::Rotate(
        Angle::from_css(input)?,
      ))),
      "matrix" => parser.parse_nested_block(|input| Ok(Transform::Matrix(
        Affine::from_css(input)?,
      ))),
      _ => Err(location.new_basic_unexpected_token_error(token.clone()).into()),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_transform_from_str() {
    let transform = Transform::from_str("translate(10, 20px)").unwrap();

    assert_eq!(
      transform,
      Transform::Translate(LengthUnit::Px(10.0), LengthUnit::Px(20.0))
    );
  }

  #[test]
  fn test_transform_scale_from_str() {
    let transform = Transform::from_str("scale(10)").unwrap();

    assert_eq!(transform, Transform::Scale(10.0, 10.0));
  }
}
