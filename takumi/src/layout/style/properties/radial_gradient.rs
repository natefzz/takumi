use cssparser::{Parser, Token, match_ignore_ascii_case};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use ts_rs::TS;

use super::gradient_utils::{color_from_stops, resolve_stops_along_axis};
use crate::{
  layout::style::{
    Color, FromCss, Gradient, GradientStop, LengthUnit, ParseResult, ResolvedGradientStop,
  },
  rendering::RenderContext,
};

/// Horizontal keywords for center position.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, TS, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum CenterKeywordX {
  /// Align to the left edge.
  Left,
  /// Align to the horizontal center.
  Center,
  /// Align to the right edge.
  Right,
}

/// Vertical keywords for center position.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, TS, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum CenterKeywordY {
  /// Align to the top edge.
  Top,
  /// Align to the vertical center.
  Center,
  /// Align to the bottom edge.
  Bottom,
}

/// A center position component that can be a keyword or length unit.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, TS, PartialEq)]
#[serde(untagged)]
pub enum CenterPositionComponent {
  /// A horizontal keyword.
  KeywordX(CenterKeywordX),
  /// A vertical keyword.
  KeywordY(CenterKeywordY),
  /// An absolute length value.
  Length(LengthUnit),
}

impl From<CenterPositionComponent> for LengthUnit {
  fn from(component: CenterPositionComponent) -> Self {
    match component {
      CenterPositionComponent::KeywordX(keyword) => match keyword {
        CenterKeywordX::Center => Self::Percentage(50.0),
        CenterKeywordX::Left => Self::Percentage(0.0),
        CenterKeywordX::Right => Self::Percentage(100.0),
      },
      CenterPositionComponent::KeywordY(keyword) => match keyword {
        CenterKeywordY::Center => Self::Percentage(50.0),
        CenterKeywordY::Top => Self::Percentage(0.0),
        CenterKeywordY::Bottom => Self::Percentage(100.0),
      },
      CenterPositionComponent::Length(length) => length,
    }
  }
}

/// Center position for radial gradients, supporting keywords and length units.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, TS, PartialEq)]
pub struct CenterPosition(pub LengthUnit, pub LengthUnit);

impl Default for CenterPosition {
  fn default() -> Self {
    Self(LengthUnit::Percentage(50.0), LengthUnit::Percentage(50.0))
  }
}

impl CenterPosition {
  /// Resolves the center position to pixel coordinates.
  pub(crate) fn resolve_to_pixels(
    self,
    context: &RenderContext,
    width: f32,
    height: f32,
  ) -> (f32, f32) {
    let cx = self.0.resolve_to_px(context, width);
    let cy = self.1.resolve_to_px(context, height);
    (cx, cy)
  }
}

/// Represents a radial gradient.
#[derive(Debug, Clone, PartialEq, TS, Deserialize, Serialize)]
pub struct RadialGradient {
  /// The radial gradient shape
  pub shape: RadialShape,
  /// The sizing mode for the gradient
  pub size: RadialSize,
  /// Center position supporting keywords and length units
  pub center: CenterPosition,
  /// Gradient stops
  pub stops: Vec<GradientStop>,
}

/// Supported shapes for radial gradients
#[derive(Debug, Clone, Copy, PartialEq, TS, Deserialize, Serialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum RadialShape {
  /// A circle shape where radii are equal
  Circle,
  /// An ellipse shape with independent x/y radii
  #[default]
  Ellipse,
}

/// Supported size keywords for radial gradients
#[derive(Debug, Clone, Copy, PartialEq, TS, Deserialize, Serialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum RadialSize {
  /// The gradient end stops at the nearest side from the center
  ClosestSide,
  /// The gradient end stops at the farthest side from the center
  FarthestSide,
  /// The gradient end stops at the nearest corner from the center
  ClosestCorner,
  /// The gradient end stops at the farthest corner from the center
  #[default]
  FarthestCorner,
}

/// Precomputed drawing context for repeated sampling of a `RadialGradient`.
#[derive(Debug, Clone)]
pub struct RadialGradientDrawContext {
  /// Target width in pixels.
  pub width: f32,
  /// Target height in pixels.
  pub height: f32,
  /// Center X coordinate in pixels
  pub cx: f32,
  /// Center Y coordinate in pixels
  pub cy: f32,
  /// Radius X in pixels (for circle, equals radius_y)
  pub radius_x: f32,
  /// Radius Y in pixels (for circle, equals radius_x)
  pub radius_y: f32,
  /// Resolved and ordered color stops.
  pub resolved_stops: SmallVec<[ResolvedGradientStop; 4]>,
}

impl Gradient for RadialGradient {
  type DrawContext = RadialGradientDrawContext;

  fn at(&self, x: u32, y: u32, ctx: &Self::DrawContext) -> Color {
    // Fast-paths
    if ctx.resolved_stops.is_empty() {
      return Color([0, 0, 0, 0]);
    }
    if ctx.resolved_stops.len() == 1 {
      return ctx.resolved_stops[0].color;
    }

    let dx = (x as f32 - ctx.cx) / ctx.radius_x.max(1e-6);
    let dy = (y as f32 - ctx.cy) / ctx.radius_y.max(1e-6);
    let position = (dx * dx + dy * dy).sqrt() * ctx.radius_x.max(ctx.radius_y);

    color_from_stops(position, &ctx.resolved_stops)
  }

  fn to_draw_context(&self, width: f32, height: f32, context: &RenderContext) -> Self::DrawContext {
    RadialGradientDrawContext::new(self, width, height, context)
  }
}

impl RadialGradient {
  /// Resolves gradient steps into color stops with positions expressed in pixels along the radial axis.
  /// Supports non-px units when a `RenderContext` is provided.
  pub(crate) fn resolve_stops_for_radius(
    &self,
    radius_scale_px: f32,
    context: &RenderContext,
  ) -> SmallVec<[ResolvedGradientStop; 4]> {
    resolve_stops_along_axis(&self.stops, radius_scale_px, context)
  }
}

impl RadialGradientDrawContext {
  /// Builds a drawing context from a gradient and a target viewport.
  pub fn new(gradient: &RadialGradient, width: f32, height: f32, context: &RenderContext) -> Self {
    let (cx, cy) = gradient.center.resolve_to_pixels(context, width, height);

    // Distances to sides and corners
    let dx_left = cx;
    let dx_right = width - cx;
    let dy_top = cy;
    let dy_bottom = height - cy;

    let (radius_x, radius_y) = match (gradient.shape, gradient.size) {
      (RadialShape::Ellipse, RadialSize::FarthestCorner) => {
        // ellipse radii to farthest corner: take farthest side per axis
        (dx_left.max(dx_right), dy_top.max(dy_bottom))
      }
      (RadialShape::Circle, RadialSize::FarthestCorner) => {
        // distance to farthest corner
        let candidates = [
          (cx, cy),
          (cx, height - cy),
          (width - cx, cy),
          (width - cx, height - cy),
        ];
        let r = candidates
          .iter()
          .map(|(dx, dy)| (dx * dx + dy * dy).sqrt())
          .fold(0.0_f32, f32::max);
        (r, r)
      }
      // Fallbacks for other size keywords: approximate using sides
      (RadialShape::Ellipse, RadialSize::FarthestSide) => {
        (dx_left.max(dx_right), dy_top.max(dy_bottom))
      }
      (RadialShape::Ellipse, RadialSize::ClosestSide) => {
        (dx_left.min(dx_right), dy_top.min(dy_bottom))
      }
      (RadialShape::Circle, RadialSize::FarthestSide) => {
        let r = dx_left.max(dx_right).max(dy_top.max(dy_bottom));
        (r, r)
      }
      (RadialShape::Circle, RadialSize::ClosestSide) => {
        let r = dx_left.min(dx_right).min(dy_top.min(dy_bottom));
        (r, r)
      }
      // For corner sizes, use farthest-corner as sensible default
      (RadialShape::Ellipse, RadialSize::ClosestCorner) => {
        (dx_left.max(dx_right), dy_top.max(dy_bottom))
      }
      (RadialShape::Circle, RadialSize::ClosestCorner) => {
        let candidates = [
          (cx, cy),
          (cx, height - cy),
          (width - cx, cy),
          (width - cx, height - cy),
        ];
        let r = candidates
          .iter()
          .map(|(dx, dy)| (dx * dx + dy * dy).sqrt())
          .fold(f32::INFINITY, f32::min);
        (r, r)
      }
    };

    let radius_scale = match gradient.shape {
      RadialShape::Circle => radius_x.max(radius_y),
      RadialShape::Ellipse => radius_x.max(radius_y),
    };
    let resolved_stops = gradient.resolve_stops_for_radius(radius_scale.max(1e-6), context);

    RadialGradientDrawContext {
      width,
      height,
      cx,
      cy,
      radius_x,
      radius_y,
      resolved_stops,
    }
  }
}

impl<'i> FromCss<'i> for RadialGradient {
  fn from_css(input: &mut Parser<'i, '_>) -> ParseResult<'i, RadialGradient> {
    input.expect_function_matching("radial-gradient")?;

    input.parse_nested_block(|input| {
      let mut shape = RadialShape::Ellipse;
      let mut size = RadialSize::FarthestCorner;
      let mut center = CenterPosition::default();

      loop {
        if let Ok(s) = input.try_parse(RadialShape::from_css) {
          shape = s;
          continue;
        }

        if let Ok(s) = input.try_parse(RadialSize::from_css) {
          size = s;
          continue;
        }

        if input.try_parse(|i| i.expect_ident_matching("at")).is_ok() {
          center = CenterPosition::from_css(input)?;
          continue;
        }

        input.try_parse(Parser::expect_comma).ok();

        break;
      }

      // Parse at least one stop, comma-separated
      let mut stops = Vec::new();

      stops.push(GradientStop::from_css(input)?);

      while input.try_parse(Parser::expect_comma).is_ok() {
        stops.push(GradientStop::from_css(input)?);
      }

      Ok(RadialGradient {
        shape,
        size,
        center,
        stops,
      })
    })
  }
}

impl<'i> FromCss<'i> for RadialShape {
  fn from_css(input: &mut Parser<'i, '_>) -> ParseResult<'i, Self> {
    let location = input.current_source_location();
    let ident = input.expect_ident()?;

    match_ignore_ascii_case! {&ident,
      "circle" => Ok(RadialShape::Circle),
      "ellipse" => Ok(RadialShape::Ellipse),
      _ => Err(location.new_basic_unexpected_token_error(Token::Ident(ident.clone())).into()),
    }
  }
}

impl<'i> FromCss<'i> for RadialSize {
  fn from_css(input: &mut Parser<'i, '_>) -> ParseResult<'i, Self> {
    let location = input.current_source_location();
    let ident = input.expect_ident()?;

    match_ignore_ascii_case! {&ident,
      "closest-side" => Ok(RadialSize::ClosestSide),
      "farthest-side" => Ok(RadialSize::FarthestSide),
      "closest-corner" => Ok(RadialSize::ClosestCorner),
      "farthest-corner" => Ok(RadialSize::FarthestCorner),
      _ => Err(location.new_basic_unexpected_token_error(Token::Ident(ident.clone())).into()),
    }
  }
}

impl<'i> FromCss<'i> for CenterPosition {
  fn from_css(input: &mut Parser<'i, '_>) -> ParseResult<'i, Self> {
    let first = CenterPositionComponent::from_css(input)?;
    // If a second exists, parse it; otherwise, 1-value syntax means y=center
    let second = input.try_parse(CenterPositionComponent::from_css).ok();

    let (x, y) = match (first, second) {
      (CenterPositionComponent::KeywordY(_), None) => (
        CenterPositionComponent::KeywordX(CenterKeywordX::Center),
        first,
      ),
      (CenterPositionComponent::KeywordY(_), Some(second)) => (second, first),
      (x, None) => (x, CenterPositionComponent::KeywordY(CenterKeywordY::Center)),
      (x, Some(y)) => (x, y),
    };

    Ok(CenterPosition(x.into(), y.into()))
  }
}

impl<'i> FromCss<'i> for CenterPositionComponent {
  fn from_css(input: &mut Parser<'i, '_>) -> ParseResult<'i, Self> {
    if let Ok(v) = input.try_parse(LengthUnit::from_css) {
      return Ok(CenterPositionComponent::Length(v));
    }

    let location = input.current_source_location();
    let token = input.expect_ident()?;

    match_ignore_ascii_case! {
      &token,
      "left" => Ok(CenterPositionComponent::KeywordX(CenterKeywordX::Left)),
      "center" => Ok(CenterPositionComponent::KeywordX(CenterKeywordX::Center)),
      "right" => Ok(CenterPositionComponent::KeywordX(CenterKeywordX::Right)),
      "top" => Ok(CenterPositionComponent::KeywordY(CenterKeywordY::Top)),
      "bottom" => Ok(CenterPositionComponent::KeywordY(CenterKeywordY::Bottom)),
      _ => Err(location.new_basic_unexpected_token_error(Token::Ident(token.clone())).into()),
    }
  }
}

/// Proxy type for `RadialGradient` Css deserialization.
#[derive(Debug, Clone, PartialEq, TS, Deserialize)]
#[serde(untagged)]
pub(crate) enum RadialGradientValue {
  /// Represents a radial gradient.
  Structured {
    /// The shape of the gradient.
    shape: RadialShape,
    /// The size keyword of the gradient.
    size: RadialSize,
    /// The center of the gradient supporting keywords and length units.
    center: CenterPosition,
    /// The steps of the gradient.
    stops: Vec<GradientStop>,
  },
  /// Represents a CSS string.
  Css(String),
}

impl TryFrom<RadialGradientValue> for RadialGradient {
  type Error = String;

  fn try_from(value: RadialGradientValue) -> Result<Self, Self::Error> {
    match value {
      RadialGradientValue::Structured {
        shape,
        size,
        center,
        stops,
      } => Ok(RadialGradient {
        shape,
        size,
        center,
        stops,
      }),
      RadialGradientValue::Css(css) => RadialGradient::from_str(&css).map_err(|e| e.to_string()),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::layout::style::{LengthUnit, StopPosition};
  use crate::{GlobalContext, layout::Viewport, rendering::RenderContext};

  #[test]
  fn test_parse_radial_gradient_basic() {
    let gradient = RadialGradient::from_str("radial-gradient(#ff0000, #0000ff)");

    assert_eq!(
      gradient,
      Ok(RadialGradient {
        shape: RadialShape::Ellipse,
        size: RadialSize::FarthestCorner,
        center: CenterPosition(LengthUnit::Percentage(50.0), LengthUnit::Percentage(50.0)),
        stops: vec![
          GradientStop::ColorHint {
            color: Color([255, 0, 0, 255]).into(),
            hint: None,
          },
          GradientStop::ColorHint {
            color: Color([0, 0, 255, 255]).into(),
            hint: None,
          },
        ],
      })
    );
  }

  #[test]
  fn test_parse_radial_gradient_circle_farthest_side() {
    let gradient =
      RadialGradient::from_str("radial-gradient(circle farthest-side, #ff0000, #0000ff)");

    assert_eq!(
      gradient,
      Ok(RadialGradient {
        shape: RadialShape::Circle,
        size: RadialSize::FarthestSide,
        center: CenterPosition(LengthUnit::Percentage(50.0), LengthUnit::Percentage(50.0)),
        stops: vec![
          GradientStop::ColorHint {
            color: Color([255, 0, 0, 255]).into(),
            hint: None,
          },
          GradientStop::ColorHint {
            color: Color([0, 0, 255, 255]).into(),
            hint: None,
          },
        ],
      })
    );
  }

  #[test]
  fn test_parse_radial_gradient_ellipse_at_left_top() {
    let gradient =
      RadialGradient::from_str("radial-gradient(ellipse at left top, #ff0000, #0000ff)");

    assert_eq!(
      gradient,
      Ok(RadialGradient {
        shape: RadialShape::Ellipse,
        size: RadialSize::FarthestCorner,
        center: CenterPosition(LengthUnit::Percentage(0.0), LengthUnit::Percentage(0.0)),
        stops: vec![
          GradientStop::ColorHint {
            color: Color([255, 0, 0, 255]).into(),
            hint: None,
          },
          GradientStop::ColorHint {
            color: Color([0, 0, 255, 255]).into(),
            hint: None,
          },
        ],
      })
    );
  }

  #[test]
  fn test_parse_radial_gradient_size_then_position() {
    let gradient =
      RadialGradient::from_str("radial-gradient(farthest-corner at 25% 60%, #ffffff, #000000)");

    assert_eq!(
      gradient,
      Ok(RadialGradient {
        shape: RadialShape::Ellipse,
        size: RadialSize::FarthestCorner,
        center: CenterPosition(
          LengthUnit::Percentage(25.0),
          LengthUnit::Percentage(60.000004)
        ),
        stops: vec![
          GradientStop::ColorHint {
            color: Color::white().into(),
            hint: None,
          },
          GradientStop::ColorHint {
            color: Color::black().into(),
            hint: None,
          },
        ],
      })
    );
  }

  #[test]
  fn test_parse_radial_gradient_circle_farthest_side_with_stops() {
    let gradient = RadialGradient::from_str(
      "radial-gradient(circle at 25px 25px, lightgray 2%, transparent 0%)",
    );

    assert_eq!(
      gradient,
      Ok(RadialGradient {
        shape: RadialShape::Circle,
        size: RadialSize::FarthestCorner,
        center: CenterPosition(LengthUnit::Px(25.0), LengthUnit::Px(25.0)),
        stops: vec![
          GradientStop::ColorHint {
            color: Color([211, 211, 211, 255]).into(),
            hint: Some(StopPosition(LengthUnit::Percentage(2.0))),
          },
          GradientStop::ColorHint {
            color: Color::transparent().into(),
            hint: Some(StopPosition(LengthUnit::Percentage(0.0))),
          },
        ],
      })
    );
  }

  #[test]
  fn test_parse_radial_gradient_with_stop_positions() {
    let gradient =
      RadialGradient::from_str("radial-gradient(circle, #ff0000 0%, #00ff00 50%, #0000ff 100%)");

    assert_eq!(
      gradient,
      Ok(RadialGradient {
        shape: RadialShape::Circle,
        size: RadialSize::FarthestCorner,
        center: CenterPosition(LengthUnit::Percentage(50.0), LengthUnit::Percentage(50.0)),
        stops: vec![
          GradientStop::ColorHint {
            color: Color([255, 0, 0, 255]).into(),
            hint: Some(StopPosition(LengthUnit::Percentage(0.0))),
          },
          GradientStop::ColorHint {
            color: Color([0, 255, 0, 255]).into(),
            hint: Some(StopPosition(LengthUnit::Percentage(50.0))),
          },
          GradientStop::ColorHint {
            color: Color([0, 0, 255, 255]).into(),
            hint: Some(StopPosition(LengthUnit::Percentage(100.0))),
          },
        ],
      })
    );
  }

  #[test]
  fn resolve_stops_percentage_and_px_radial() {
    let gradient = RadialGradient {
      shape: RadialShape::Ellipse,
      size: RadialSize::FarthestCorner,
      center: CenterPosition(LengthUnit::Percentage(50.0), LengthUnit::Percentage(50.0)),
      stops: vec![
        GradientStop::ColorHint {
          color: Color::black().into(),
          hint: Some(StopPosition(LengthUnit::Percentage(0.0))),
        },
        GradientStop::ColorHint {
          color: Color::black().into(),
          hint: Some(StopPosition(LengthUnit::Percentage(50.0))),
        },
        GradientStop::ColorHint {
          color: Color::black().into(),
          hint: Some(StopPosition(LengthUnit::Px(100.0))),
        },
      ],
    };

    let context = GlobalContext::default();
    let render_context = RenderContext::new(&context, Viewport::new(200, 100), Default::default());
    let resolved =
      gradient.resolve_stops_for_radius(render_context.viewport.width as f32, &render_context);

    assert_eq!(resolved.len(), 3);
    assert!((resolved[0].position - 0.0).abs() < 1e-3);
    assert_eq!(resolved[1].position, resolved[2].position);
  }

  #[test]
  fn resolve_stops_equal_positions_distributed_radial() {
    let gradient = RadialGradient {
      shape: RadialShape::Ellipse,
      size: RadialSize::FarthestCorner,
      center: CenterPosition(LengthUnit::Percentage(50.0), LengthUnit::Percentage(50.0)),
      stops: vec![
        GradientStop::ColorHint {
          color: Color::black().into(),
          hint: Some(StopPosition(LengthUnit::Px(0.0))),
        },
        GradientStop::ColorHint {
          color: Color::black().into(),
          hint: Some(StopPosition(LengthUnit::Px(0.0))),
        },
        GradientStop::ColorHint {
          color: Color::black().into(),
          hint: Some(StopPosition(LengthUnit::Px(0.0))),
        },
      ],
    };

    let context = GlobalContext::default();
    let render_context = RenderContext::new(&context, Viewport::new(200, 100), Default::default());
    let resolved =
      gradient.resolve_stops_for_radius(render_context.viewport.width as f32, &render_context);

    assert_eq!(resolved.len(), 3);
    assert!(resolved[0].position >= 0.0);
    assert!(resolved[1].position >= resolved[0].position);
    assert!(resolved[2].position >= resolved[1].position);
  }
}
