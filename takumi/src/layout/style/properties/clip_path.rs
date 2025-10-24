use cssparser::{Parser, ParserInput, Token, match_ignore_ascii_case};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::layout::style::{FromCss, LengthUnit, ParseResult};

/// Represents the fill rule used for determining the interior of shapes.
///
/// Corresponds to the SVG fill-rule attribute and is used in polygon(), path(), and shape() functions.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, TS, PartialEq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum FillRule {
  /// The default rule - counts the number of times a ray from the point crosses the shape's edges
  #[default]
  NonZero,
  /// Counts the total number of crossings - if even, the point is outside
  EvenOdd,
}

/// Represents radius values for circle() and ellipse() functions.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, TS, PartialEq)]
#[serde(rename_all = "kebab-case")]
#[derive(Default)]
pub enum ShapeRadius {
  /// A specific length value
  Length(LengthUnit),
  /// A percentage of the reference box
  Percentage(f32),
  /// Uses the length from the center to the closest side of the reference box
  #[default]
  ClosestSide,
  /// Uses the length from the center to the farthest side of the reference box
  FarthestSide,
}

/// Represents a position for circle() and ellipse() functions.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, TS, PartialEq)]
pub struct ShapePosition {
  /// X-axis position component
  pub x: LengthUnit,
  /// Y-axis position component
  pub y: LengthUnit,
}

impl Default for ShapePosition {
  fn default() -> Self {
    Self {
      x: LengthUnit::Percentage(50.0),
      y: LengthUnit::Percentage(50.0),
    }
  }
}

/// Represents an inset() rectangle shape.
///
/// The inset() function creates an inset rectangle, with its size defined by the offset distance
/// of each of the four sides of its container and, optionally, rounded corners.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct InsetShape {
  /// Top inset distance
  pub top: LengthUnit,
  /// Right inset distance
  pub right: LengthUnit,
  /// Bottom inset distance
  pub bottom: LengthUnit,
  /// Left inset distance
  pub left: LengthUnit,
  /// Optional border radius for rounded corners
  pub border_radius: Option<Box<super::Sides<LengthUnit>>>,
}

/// Represents a circle() shape.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct CircleShape {
  /// The radius of the circle
  pub radius: ShapeRadius,
  /// The center position of the circle
  pub position: ShapePosition,
}

/// Represents an ellipse() shape.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct EllipseShape {
  /// The horizontal radius
  pub radius_x: ShapeRadius,
  /// The vertical radius
  pub radius_y: ShapeRadius,
  /// The center position of the ellipse
  pub position: ShapePosition,
}

/// Represents a single coordinate pair in a polygon.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct PolygonCoordinate {
  /// X coordinate
  pub x: LengthUnit,
  /// Y coordinate
  pub y: LengthUnit,
}

/// Represents a polygon() shape.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct PolygonShape {
  /// The fill rule to use
  pub fill_rule: FillRule,
  /// List of coordinate pairs defining the polygon vertices
  pub coordinates: Vec<PolygonCoordinate>,
}

/// Represents a path() shape using an SVG path string.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct PathShape {
  /// The fill rule to use
  pub fill_rule: FillRule,
  /// SVG path data string
  pub path_data: String,
}

/// Represents a basic shape function for clip-path.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum BasicShape {
  /// inset() function
  Inset(InsetShape),
  /// circle() function
  Circle(CircleShape),
  /// ellipse() function
  Ellipse(EllipseShape),
  /// polygon() function
  Polygon(PolygonShape),
  /// path() function
  Path(PathShape),
}

/// Represents the clip-path property value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS, Default)]
#[ts(as = "ClipPathValue")]
#[serde(try_from = "ClipPathValue")]
pub enum ClipPath {
  /// No clipping
  #[default]
  None,
  /// Basic shape function
  Shape(BasicShape),
}

/// Proxy type for CSS deserialization that accepts either structured data or CSS strings.
#[derive(Debug, Clone, Deserialize, TS)]
#[serde(untagged)]
pub enum ClipPathValue {
  /// A structured basic shape
  Shape(BasicShape),
  /// Raw CSS string to be parsed
  Css(String),
}

impl TryFrom<ClipPathValue> for ClipPath {
  type Error = String;

  fn try_from(value: ClipPathValue) -> Result<Self, Self::Error> {
    match value {
      ClipPathValue::Shape(shape) => Ok(ClipPath::Shape(shape)),
      ClipPathValue::Css(css) => {
        let mut input = ParserInput::new(&css);
        let mut parser = Parser::new(&mut input);

        ClipPath::from_css(&mut parser).map_err(|e| e.to_string())
      }
    }
  }
}

impl<'i> FromCss<'i> for FillRule {
  fn from_css(parser: &mut Parser<'i, '_>) -> ParseResult<'i, Self> {
    let location = parser.current_source_location();
    let ident = parser.expect_ident()?;

    match_ignore_ascii_case! { &ident,
      "nonzero" => Ok(FillRule::NonZero),
      "evenodd" => Ok(FillRule::EvenOdd),
      _ => Err(location.new_basic_unexpected_token_error(Token::Ident(ident.clone())).into())
    }
  }
}

impl<'i> FromCss<'i> for ShapeRadius {
  fn from_css(parser: &mut Parser<'i, '_>) -> ParseResult<'i, Self> {
    let location = parser.current_source_location();

    // Try parsing as length first
    if let Ok(length) = parser.try_parse(LengthUnit::from_css) {
      return match length {
        LengthUnit::Percentage(p) => Ok(ShapeRadius::Percentage(p)),
        length => Ok(ShapeRadius::Length(length)),
      };
    }

    // Try parsing keywords
    let ident = parser.expect_ident()?;
    match_ignore_ascii_case! { &ident,
      "closest-side" => Ok(ShapeRadius::ClosestSide),
      "farthest-side" => Ok(ShapeRadius::FarthestSide),
      _ => Err(location.new_basic_unexpected_token_error(Token::Ident(ident.clone())).into())
    }
  }
}

impl<'i> FromCss<'i> for ShapePosition {
  fn from_css(parser: &mut Parser<'i, '_>) -> ParseResult<'i, Self> {
    let first = LengthUnit::from_css(parser)?;

    // If there's a second value, parse it; otherwise default to 50%
    let second = parser
      .try_parse(LengthUnit::from_css)
      .unwrap_or(LengthUnit::Percentage(50.0));

    Ok(ShapePosition {
      x: first,
      y: second,
    })
  }
}

impl<'i> FromCss<'i> for PolygonCoordinate {
  fn from_css(parser: &mut Parser<'i, '_>) -> ParseResult<'i, Self> {
    let x = LengthUnit::from_css(parser)?;
    let y = LengthUnit::from_css(parser)?;

    Ok(PolygonCoordinate { x, y })
  }
}

impl<'i> FromCss<'i> for ClipPath {
  fn from_css(parser: &mut Parser<'i, '_>) -> ParseResult<'i, Self> {
    let location = parser.current_source_location();
    let token = parser.next()?;

    match token {
      Token::Ident(ident) => {
        match_ignore_ascii_case! { &ident,
          "none" => Ok(ClipPath::None),
          _ => Err(location.new_basic_unexpected_token_error(token.clone()).into())
        }
      }
      Token::Function(function) => {
        match_ignore_ascii_case! { &function,
          "inset" => parser.parse_nested_block(|input| {
            let top = LengthUnit::from_css(input)?;
            let right = input.try_parse(LengthUnit::from_css).unwrap_or(top);
            let bottom = input.try_parse(LengthUnit::from_css).unwrap_or(top);
            let left = input.try_parse(LengthUnit::from_css).unwrap_or(right);

            // Parse border radius with "round" keyword
            let border_radius = if input.try_parse(|input| input.expect_ident_matching("round")).is_ok() {
              // Parse 1-4 values for border radius (similar to CSS border-radius syntax)
              let first = LengthUnit::from_css(input)?;
              let second = input.try_parse(LengthUnit::from_css).unwrap_or(first);
              let third = input.try_parse(LengthUnit::from_css).unwrap_or(first);
              let fourth = input.try_parse(LengthUnit::from_css).unwrap_or(second);

              let radii = super::Sides([first, second, third, fourth]);
              Some(Box::new(radii))
            } else {
              None
            };

            let inset_shape = InsetShape {
              top,
              right,
              bottom,
              left,
              border_radius,
            };

            Ok(ClipPath::Shape(BasicShape::Inset(inset_shape)))
          }),
          "circle" => parser.parse_nested_block(|input| {
            let radius = input.try_parse(ShapeRadius::from_css).unwrap_or_default();

            let position = if input.try_parse(|input| input.expect_ident_matching("at")).is_ok() {
              ShapePosition::from_css(input)?
            } else {
              ShapePosition::default()
            };

            let circle_shape = CircleShape { radius, position };
            Ok(ClipPath::Shape(BasicShape::Circle(circle_shape)))
          }),
          "ellipse" => parser.parse_nested_block(|input| {
            let radius_x = ShapeRadius::from_css(input)?;
            let radius_y = input.try_parse(ShapeRadius::from_css).unwrap_or_default();

            let position = if input.try_parse(|input| input.expect_ident_matching("at")).is_ok() {
              ShapePosition::from_css(input)?
            } else {
              ShapePosition::default()
            };

            let ellipse_shape = EllipseShape { radius_x, radius_y, position };
            Ok(ClipPath::Shape(BasicShape::Ellipse(ellipse_shape)))
          }),
          "polygon" => parser.parse_nested_block(|input| {
            let fill_rule = input.try_parse(FillRule::from_css).ok();
            if fill_rule.is_some() {
              input.expect_comma()?;
            }

            let mut coordinates = Vec::new();

            // Parse first coordinate pair
            coordinates.push(PolygonCoordinate::from_css(input)?);

            // Parse remaining coordinate pairs
            while input.try_parse(Parser::expect_comma).is_ok() {
              coordinates.push(PolygonCoordinate::from_css(input)?);
            }

            let polygon_shape = PolygonShape {
              fill_rule: fill_rule.unwrap_or_default(),
              coordinates,
            };

            Ok(ClipPath::Shape(BasicShape::Polygon(polygon_shape)))
          }),
          "path" => parser.parse_nested_block(|input| {
            let fill_rule = input.try_parse(FillRule::from_css).ok();
            if fill_rule.is_some() {
              input.expect_comma()?;
            }

            let path_data = input.expect_string()?.to_string();

            let path_shape = PathShape {
              fill_rule: fill_rule.unwrap_or_default(),
              path_data,
            };

            Ok(ClipPath::Shape(BasicShape::Path(path_shape)))
          }),
          _ => Err(location.new_basic_unexpected_token_error(token.clone()).into())
        }
      }
      _ => Err(
        location
          .new_basic_unexpected_token_error(token.clone())
          .into(),
      ),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use cssparser::{Parser, ParserInput};

  fn parse_clip_path_str(css: &str) -> ClipPath {
    let mut input = ParserInput::new(css);
    let mut parser = Parser::new(&mut input);
    ClipPath::from_css(&mut parser).unwrap()
  }

  #[test]
  fn test_parse_none() {
    let result = parse_clip_path_str("none");
    assert_eq!(result, ClipPath::None);
  }

  #[test]
  fn test_parse_inset_simple() {
    let result = parse_clip_path_str("inset(10px)");
    if let ClipPath::Shape(BasicShape::Inset(inset)) = result {
      assert_eq!(inset.top, LengthUnit::Px(10.0));
      assert_eq!(inset.right, LengthUnit::Px(10.0));
      assert_eq!(inset.bottom, LengthUnit::Px(10.0));
      assert_eq!(inset.left, LengthUnit::Px(10.0));
    } else {
      panic!("Expected inset shape");
    }
  }

  #[test]
  fn test_parse_inset_four_values() {
    let result = parse_clip_path_str("inset(10px 20px 30px 40px)");
    if let ClipPath::Shape(BasicShape::Inset(inset)) = result {
      assert_eq!(inset.top, LengthUnit::Px(10.0));
      assert_eq!(inset.right, LengthUnit::Px(20.0));
      assert_eq!(inset.bottom, LengthUnit::Px(30.0));
      assert_eq!(inset.left, LengthUnit::Px(40.0));
      assert_eq!(inset.border_radius, None);
    } else {
      panic!("Expected inset shape");
    }
  }

  #[test]
  fn test_parse_inset_with_border_radius() {
    let result = parse_clip_path_str("inset(10px round 5px)");
    if let ClipPath::Shape(BasicShape::Inset(inset)) = result {
      assert_eq!(inset.top, LengthUnit::Px(10.0));
      assert_eq!(inset.right, LengthUnit::Px(10.0));
      assert_eq!(inset.bottom, LengthUnit::Px(10.0));
      assert_eq!(inset.left, LengthUnit::Px(10.0));

      let border_radius = inset.border_radius.expect("Should have border radius");
      assert_eq!(border_radius.0[0], LengthUnit::Px(5.0)); // top-left
      assert_eq!(border_radius.0[1], LengthUnit::Px(5.0)); // top-right
      assert_eq!(border_radius.0[2], LengthUnit::Px(5.0)); // bottom-right
      assert_eq!(border_radius.0[3], LengthUnit::Px(5.0)); // bottom-left
    } else {
      panic!("Expected inset shape");
    }
  }

  #[test]
  fn test_parse_inset_with_complex_border_radius() {
    let result = parse_clip_path_str("inset(10px 20px 30px 40px round 5px 10px 15px 20px)");
    if let ClipPath::Shape(BasicShape::Inset(inset)) = result {
      assert_eq!(inset.top, LengthUnit::Px(10.0));
      assert_eq!(inset.right, LengthUnit::Px(20.0));
      assert_eq!(inset.bottom, LengthUnit::Px(30.0));
      assert_eq!(inset.left, LengthUnit::Px(40.0));

      let border_radius = inset.border_radius.expect("Should have border radius");
      assert_eq!(border_radius.0[0], LengthUnit::Px(5.0)); // top-left
      assert_eq!(border_radius.0[1], LengthUnit::Px(10.0)); // top-right
      assert_eq!(border_radius.0[2], LengthUnit::Px(15.0)); // bottom-right
      assert_eq!(border_radius.0[3], LengthUnit::Px(20.0)); // bottom-left
    } else {
      panic!("Expected inset shape");
    }
  }

  #[test]
  fn test_parse_circle_simple() {
    let result = parse_clip_path_str("circle(50px)");
    if let ClipPath::Shape(BasicShape::Circle(circle)) = result {
      assert_eq!(circle.radius, ShapeRadius::Length(LengthUnit::Px(50.0)));
      assert_eq!(circle.position.x, LengthUnit::Percentage(50.0));
      assert_eq!(circle.position.y, LengthUnit::Percentage(50.0));
    } else {
      panic!("Expected circle shape");
    }
  }

  #[test]
  fn test_parse_circle_with_position() {
    let result = parse_clip_path_str("circle(50px at 25% 75%)");
    if let ClipPath::Shape(BasicShape::Circle(circle)) = result {
      assert_eq!(circle.radius, ShapeRadius::Length(LengthUnit::Px(50.0)));
      assert_eq!(circle.position.x, LengthUnit::Percentage(25.0));
      assert_eq!(circle.position.y, LengthUnit::Percentage(75.0));
    } else {
      panic!("Expected circle shape");
    }
  }

  #[test]
  fn test_parse_circle_default_radius() {
    let result = parse_clip_path_str("circle(at 25% 75%)");
    if let ClipPath::Shape(BasicShape::Circle(circle)) = result {
      assert_eq!(circle.radius, ShapeRadius::ClosestSide);
      assert_eq!(circle.position.x, LengthUnit::Percentage(25.0));
      assert_eq!(circle.position.y, LengthUnit::Percentage(75.0));
    } else {
      panic!("Expected circle shape");
    }
  }

  #[test]
  fn test_parse_ellipse_simple() {
    let result = parse_clip_path_str("ellipse(50px 30px)");
    if let ClipPath::Shape(BasicShape::Ellipse(ellipse)) = result {
      assert_eq!(ellipse.radius_x, ShapeRadius::Length(LengthUnit::Px(50.0)));
      assert_eq!(ellipse.radius_y, ShapeRadius::Length(LengthUnit::Px(30.0)));
      assert_eq!(ellipse.position.x, LengthUnit::Percentage(50.0));
      assert_eq!(ellipse.position.y, LengthUnit::Percentage(50.0));
    } else {
      panic!("Expected ellipse shape");
    }
  }

  #[test]
  fn test_parse_ellipse_with_position() {
    let result = parse_clip_path_str("ellipse(50px 30px at 25% 75%)");
    if let ClipPath::Shape(BasicShape::Ellipse(ellipse)) = result {
      assert_eq!(ellipse.radius_x, ShapeRadius::Length(LengthUnit::Px(50.0)));
      assert_eq!(ellipse.radius_y, ShapeRadius::Length(LengthUnit::Px(30.0)));
      assert_eq!(ellipse.position.x, LengthUnit::Percentage(25.0));
      assert_eq!(ellipse.position.y, LengthUnit::Percentage(75.0));
    } else {
      panic!("Expected ellipse shape");
    }
  }

  #[test]
  fn test_parse_polygon_triangle() {
    let result = parse_clip_path_str("polygon(50% 0%, 0% 100%, 100% 100%)");
    if let ClipPath::Shape(BasicShape::Polygon(polygon)) = result {
      assert_eq!(polygon.fill_rule, FillRule::NonZero);
      assert_eq!(polygon.coordinates.len(), 3);

      assert_eq!(polygon.coordinates[0].x, LengthUnit::Percentage(50.0));
      assert_eq!(polygon.coordinates[0].y, LengthUnit::Percentage(0.0));
      assert_eq!(polygon.coordinates[1].x, LengthUnit::Percentage(0.0));
      assert_eq!(polygon.coordinates[1].y, LengthUnit::Percentage(100.0));
      assert_eq!(polygon.coordinates[2].x, LengthUnit::Percentage(100.0));
      assert_eq!(polygon.coordinates[2].y, LengthUnit::Percentage(100.0));
    } else {
      panic!("Expected polygon shape");
    }
  }

  #[test]
  fn test_parse_polygon_with_fill_rule() {
    let result = parse_clip_path_str("polygon(evenodd, 50% 0%, 0% 100%, 100% 100%)");
    if let ClipPath::Shape(BasicShape::Polygon(polygon)) = result {
      assert_eq!(polygon.fill_rule, FillRule::EvenOdd);
      assert_eq!(polygon.coordinates.len(), 3);
    } else {
      panic!("Expected polygon shape");
    }
  }

  #[test]
  fn test_parse_path() {
    let result = parse_clip_path_str("path('M 10 10 L 90 90')");
    if let ClipPath::Shape(BasicShape::Path(path)) = result {
      assert_eq!(path.fill_rule, FillRule::NonZero);
      assert_eq!(path.path_data, "M 10 10 L 90 90");
    } else {
      panic!("Expected path shape");
    }
  }

  #[test]
  fn test_parse_path_with_fill_rule() {
    let result = parse_clip_path_str("path(evenodd, 'M 10 10 L 90 90')");
    if let ClipPath::Shape(BasicShape::Path(path)) = result {
      assert_eq!(path.fill_rule, FillRule::EvenOdd);
      assert_eq!(path.path_data, "M 10 10 L 90 90");
    } else {
      panic!("Expected path shape");
    }
  }

  #[test]
  fn test_parse_circle_percentage_radius() {
    let result = parse_clip_path_str("circle(50%)");
    if let ClipPath::Shape(BasicShape::Circle(circle)) = result {
      assert_eq!(circle.radius, ShapeRadius::Percentage(50.0));
    } else {
      panic!("Expected circle shape");
    }
  }

  #[test]
  fn test_parse_circle_closest_side() {
    let result = parse_clip_path_str("circle(closest-side)");
    if let ClipPath::Shape(BasicShape::Circle(circle)) = result {
      assert_eq!(circle.radius, ShapeRadius::ClosestSide);
    } else {
      panic!("Expected circle shape");
    }
  }

  #[test]
  fn test_parse_circle_farthest_side() {
    let result = parse_clip_path_str("circle(farthest-side)");
    if let ClipPath::Shape(BasicShape::Circle(circle)) = result {
      assert_eq!(circle.radius, ShapeRadius::FarthestSide);
    } else {
      panic!("Expected circle shape");
    }
  }
}
