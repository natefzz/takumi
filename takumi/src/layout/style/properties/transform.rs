use std::ops::{Mul, MulAssign};

use cssparser::{Parser, Token, match_ignore_ascii_case};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
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

/// | a b x |
/// | c d y |
/// | 0 0 1 |
#[derive(Debug, Clone, Copy, TS, Default, PartialEq)]
#[ts(type = "string")]
pub struct Affine {
  /// Horizontal scaling / cosine of rotation
  pub a: f32,
  /// Vertical shear / sine of rotation
  pub b: f32,
  /// Horizontal shear / negative sine of rotation
  pub c: f32,
  /// Vertical scaling / cosine of rotation
  pub d: f32,
  /// Horizontal translation (always orthogonal regardless of rotation)
  pub x: f32,
  /// Vertical translation (always orthogonal regardless of rotation)
  pub y: f32,
}

impl Mul<Affine> for Affine {
  type Output = Affine;

  fn mul(self, rhs: Affine) -> Self::Output {
    Affine {
      a: self.a * rhs.a + self.b * rhs.c,
      b: self.a * rhs.b + self.b * rhs.d,
      c: self.c * rhs.a + self.d * rhs.c,
      d: self.c * rhs.b + self.d * rhs.d,
      x: self.x * rhs.a + self.y * rhs.c + rhs.x,
      y: self.x * rhs.b + self.y * rhs.d + rhs.y,
    }
  }
}

impl MulAssign<Affine> for Affine {
  fn mul_assign(&mut self, rhs: Affine) {
    *self = *self * rhs;
  }
}

impl Affine {
  /// Returns the identity transform
  pub const IDENTITY: Self = Self {
    a: 1.0,
    b: 0.0,
    c: 0.0,
    d: 1.0,
    x: 0.0,
    y: 0.0,
  };

  /// Returns true if the transform is the identity transform
  pub fn is_identity(self) -> bool {
    self == Self::IDENTITY
  }

  /// Decomposes the translation part of the transform
  pub fn decompose_translation(self) -> Point<f32> {
    Point {
      x: self.x,
      y: self.y,
    }
  }

  /// Returns true if the transform is only a translation
  pub(crate) fn only_translation(self) -> bool {
    self.a == Self::IDENTITY.a
      && self.b == Self::IDENTITY.b
      && self.c == Self::IDENTITY.c
      && self.d == Self::IDENTITY.d
  }

  /// Creates a new rotation transform
  pub fn rotation(angle: Angle) -> Self {
    let (sin, cos) = angle.to_radians().sin_cos();

    Self {
      a: cos,
      b: sin,
      c: -sin,
      d: cos,
      x: 0.0,
      y: 0.0,
    }
  }

  /// Creates a new translation transform
  pub const fn translation(x: f32, y: f32) -> Self {
    Self {
      x,
      y,
      ..Self::IDENTITY
    }
  }

  /// Creates a new scale transform
  pub const fn scale(x: f32, y: f32) -> Self {
    Self {
      a: x,
      b: 0.0,
      c: 0.0,
      d: y,
      x: 0.0,
      y: 0.0,
    }
  }

  /// Transforms a point by the transform
  #[inline(always)]
  pub fn transform_point(self, point: Point<f32>) -> Point<f32> {
    Point {
      x: self.a * point.x + self.c * point.y + self.x,
      y: self.b * point.x + self.d * point.y + self.y,
    }
  }

  /// Creates a new skew transform
  pub fn skew(x: Angle, y: Angle) -> Self {
    let tanx = x.to_radians().tan();
    let tany = y.to_radians().tan();

    Self {
      a: 1.0,
      b: tany,
      c: tanx,
      d: 1.0,
      x: 0.0,
      y: 0.0,
    }
  }

  /// Calculates the determinant of the transform
  #[inline(always)]
  pub fn determinant(self) -> f32 {
    self.a * self.d - self.b * self.c
  }

  /// Returns true if the transform is invertible
  #[inline(always)]
  pub fn is_invertible(self) -> bool {
    self.determinant().abs() > f32::EPSILON
  }

  /// Inverts the transform, returns `None` if the transform is not invertible
  pub fn invert(self) -> Option<Self> {
    let det = self.determinant();
    if det.abs() < f32::EPSILON {
      return None;
    }

    Some(Self {
      a: self.d / det,
      b: self.b / -det,
      c: self.c / -det,
      d: self.a / det,
      x: (self.d * self.x - self.c * self.y) / -det,
      y: (self.b * self.x - self.a * self.y) / det,
    })
  }
}

impl From<Affine> for zeno::Transform {
  fn from(affine: Affine) -> Self {
    zeno::Transform::new(affine.a, affine.b, affine.c, affine.d, affine.x, affine.y)
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
    let mut string = String::new();

    string.push_str(&self.a.to_string());
    string.push(',');
    string.push_str(&self.b.to_string());
    string.push(',');
    string.push_str(&self.c.to_string());
    string.push(',');
    string.push_str(&self.d.to_string());
    string.push(',');
    string.push_str(&self.x.to_string());
    string.push(',');
    string.push_str(&self.y.to_string());

    serializer.serialize_str(&string)
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

    Ok(Affine { a, b, c, d, x, y })
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
    let mut instance = Affine::IDENTITY;

    for transform in self.0.iter() {
      match *transform {
        Transform::Translate(x_length, y_length) => {
          instance *= Affine::translation(
            x_length.resolve_to_px(context, border_box.width),
            y_length.resolve_to_px(context, border_box.height),
          );
        }
        Transform::Scale(x_scale, y_scale) => {
          instance *= Affine::scale(x_scale, y_scale);
        }
        Transform::Rotate(angle) => {
          instance *= Affine::rotation(angle);
        }
        Transform::Skew(x_angle, y_angle) => {
          instance *= Affine::skew(x_angle, y_angle);
        }
        Transform::Matrix(affine) => {
          instance *= affine;
        }
      }
    }

    instance
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
