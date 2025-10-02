use cssparser::{Parser, ParserInput, Token, match_ignore_ascii_case};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use ts_rs::TS;

use super::gradient_utils::{color_from_stops, resolve_stops_along_axis};
use crate::{
  layout::style::{
    Color, FromCss, Gradient, GradientStop, ParseResult, ResolvedGradientStop,
    parse_length_percentage,
  },
  rendering::RenderContext,
};

/// Represents a radial gradient.
#[derive(Debug, Clone, PartialEq, TS, Deserialize, Serialize)]
pub struct RadialGradient {
  /// The radial gradient shape
  pub shape: RadialShape,
  /// The sizing mode for the gradient
  pub size: RadialSize,
  /// Center position in normalized coordinates [0.0, 1.0]
  pub center: (f32, f32),
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
    let cx = (gradient.center.0.clamp(0.0, 1.0)) * width;
    let cy = (gradient.center.1.clamp(0.0, 1.0)) * height;

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
      let mut center = (0.5_f32, 0.5_f32);

      // Optional prelude: [<ending-shape> || <size>]? [at <position>]? ,
      // Try to parse up to one shape and one size in any order
      let mut parsed_any_prelude = false;

      // Helper closure to try parse shape or size once
      let mut try_parse_shape_or_size = |input: &mut Parser<'i, '_>| -> bool {
        if let Ok(s) = input.try_parse(RadialShape::from_css) {
          shape = s;
          return true;
        }
        if let Ok(s) = input.try_parse(RadialSize::from_css) {
          size = s;
          return true;
        }
        false
      };

      // Attempt to parse any combination
      parsed_any_prelude |= try_parse_shape_or_size(input);
      parsed_any_prelude |= try_parse_shape_or_size(input);

      // Optional position: at <position>
      if input.try_parse(|i| i.expect_ident_matching("at")).is_ok() {
        center = RadialPosition::from_css(input)?;
        parsed_any_prelude = true;
      }

      // If there was any prelude, expect a comma separator before stops when present in the source.
      // The css syntax requires a comma between prelude and first stop.
      // If the author omitted prelude, the next token should already be a color or percentage before comma.
      if parsed_any_prelude {
        // Allow cases where the next is a comma, otherwise stops may follow immediately
        let _ = input.try_parse(Parser::expect_comma);
      }

      // Parse at least one stop, comma-separated
      let mut steps = Vec::new();
      steps.push(GradientStop::from_css(input)?);
      while input.try_parse(Parser::expect_comma).is_ok() {
        steps.push(GradientStop::from_css(input)?);
      }

      Ok(RadialGradient {
        shape,
        size,
        center,
        stops: steps,
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

/// Represents radial position keywords or percentages
#[derive(Debug, Clone, Copy, PartialEq)]
struct RadialPosition;

impl RadialPosition {
  /// Parses a position definition into normalized (x, y) in [0,1]
  fn from_css<'i>(input: &mut Parser<'i, '_>) -> ParseResult<'i, (f32, f32)> {
    // Try keyword-based positions first
    let mut x: Option<f32> = None;
    let mut y: Option<f32> = None;

    // Helper to parse a single keyword to percentage
    let parse_keyword = |ident: &str| -> Option<(Option<f32>, Option<f32>)> {
      match ident.to_ascii_lowercase().as_str() {
        "left" => Some((Some(0.0), None)),
        "right" => Some((Some(1.0), None)),
        "center" => Some((None, None)),
        "top" => Some((None, Some(0.0))),
        "bottom" => Some((None, Some(1.0))),
        _ => None,
      }
    };

    // Try to read up to two idents
    for _ in 0..2 {
      let parsed = input
        .try_parse(
          |i| -> Result<(), cssparser::ParseError<'i, cssparser::BasicParseError<'i>>> {
            let location = i.current_source_location();
            let ident = i.expect_ident()?;
            if let Some((maybe_x, maybe_y)) = parse_keyword(ident) {
              if let Some(v) = maybe_x {
                x = Some(v);
              }
              if let Some(v) = maybe_y {
                y = Some(v);
              }
              Ok(())
            } else {
              Err(
                location
                  .new_basic_unexpected_token_error(Token::Ident(ident.clone()))
                  .into(),
              )
            }
          },
        )
        .is_ok();

      if !parsed {
        break;
      }
    }

    if x.is_some() || y.is_some() {
      return Ok((x.unwrap_or(0.5), y.unwrap_or(0.5)));
    }

    // Numeric/percentage path: parse one or two length/percentage values
    if let Ok(px) = input.try_parse(parse_length_percentage) {
      let py = input.try_parse(parse_length_percentage).ok().unwrap_or(0.5);
      return Ok((px, py));
    }

    // Default center
    Ok((0.5, 0.5))
  }
}

/// Proxy type for `RadialGradient` Css deserialization.
#[derive(Debug, Clone, PartialEq, TS, Deserialize)]
#[serde(untagged)]
pub enum RadialGradientValue {
  /// Represents a radial gradient.
  Structured {
    /// The shape of the gradient.
    shape: RadialShape,
    /// The size keyword of the gradient.
    size: RadialSize,
    /// The center of the gradient in normalized [0,1] coords.
    center: (f32, f32),
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
      RadialGradientValue::Css(css) => {
        let mut input = ParserInput::new(&css);
        let mut parser = Parser::new(&mut input);

        RadialGradient::from_css(&mut parser).map_err(|e| e.to_string())
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::layout::DEFAULT_FONT_SIZE;
  use crate::layout::style::{Affine, InheritedStyle, LengthUnit, StopPosition};
  use crate::{GlobalContext, layout::Viewport, rendering::RenderContext};

  #[test]
  fn test_parse_radial_gradient_basic() {
    let mut input = ParserInput::new("radial-gradient(#ff0000, #0000ff)");
    let mut parser = Parser::new(&mut input);
    let gradient = RadialGradient::from_css(&mut parser);

    assert_eq!(
      gradient,
      Ok(RadialGradient {
        shape: RadialShape::Ellipse,
        size: RadialSize::FarthestCorner,
        center: (0.5, 0.5),
        stops: vec![
          GradientStop::ColorHint {
            color: Color([255, 0, 0, 255]),
            hint: None,
          },
          GradientStop::ColorHint {
            color: Color([0, 0, 255, 255]),
            hint: None,
          },
        ],
      })
    );
  }

  #[test]
  fn test_parse_radial_gradient_circle_farthest_side() {
    let mut input = ParserInput::new("radial-gradient(circle farthest-side, #ff0000, #0000ff)");
    let mut parser = Parser::new(&mut input);
    let gradient = RadialGradient::from_css(&mut parser);

    assert_eq!(
      gradient,
      Ok(RadialGradient {
        shape: RadialShape::Circle,
        size: RadialSize::FarthestSide,
        center: (0.5, 0.5),
        stops: vec![
          GradientStop::ColorHint {
            color: Color([255, 0, 0, 255]),
            hint: None,
          },
          GradientStop::ColorHint {
            color: Color([0, 0, 255, 255]),
            hint: None,
          },
        ],
      })
    );
  }

  #[test]
  fn test_parse_radial_gradient_ellipse_at_left_top() {
    let mut input = ParserInput::new("radial-gradient(ellipse at left top, #ff0000, #0000ff)");
    let mut parser = Parser::new(&mut input);
    let gradient = RadialGradient::from_css(&mut parser);

    assert_eq!(
      gradient,
      Ok(RadialGradient {
        shape: RadialShape::Ellipse,
        size: RadialSize::FarthestCorner,
        center: (0.0, 0.0),
        stops: vec![
          GradientStop::ColorHint {
            color: Color([255, 0, 0, 255]),
            hint: None,
          },
          GradientStop::ColorHint {
            color: Color([0, 0, 255, 255]),
            hint: None,
          },
        ],
      })
    );
  }

  #[test]
  fn test_parse_radial_gradient_size_then_position() {
    let mut input =
      ParserInput::new("radial-gradient(farthest-corner at 25% 60%, #ffffff, #000000)");
    let mut parser = Parser::new(&mut input);
    let gradient = RadialGradient::from_css(&mut parser);

    assert_eq!(
      gradient,
      Ok(RadialGradient {
        shape: RadialShape::Ellipse,
        size: RadialSize::FarthestCorner,
        center: (0.25, 0.6),
        stops: vec![
          GradientStop::ColorHint {
            color: Color::white(),
            hint: None,
          },
          GradientStop::ColorHint {
            color: Color::black(),
            hint: None,
          },
        ],
      })
    );
  }

  #[test]
  fn test_parse_radial_gradient_with_stop_positions() {
    let mut input =
      ParserInput::new("radial-gradient(circle, #ff0000 0%, #00ff00 50%, #0000ff 100%)");
    let mut parser = Parser::new(&mut input);
    let gradient = RadialGradient::from_css(&mut parser);

    assert_eq!(
      gradient,
      Ok(RadialGradient {
        shape: RadialShape::Circle,
        size: RadialSize::FarthestCorner,
        center: (0.5, 0.5),
        stops: vec![
          GradientStop::ColorHint {
            color: Color([255, 0, 0, 255]),
            hint: Some(StopPosition(LengthUnit::Percentage(0.0))),
          },
          GradientStop::ColorHint {
            color: Color([0, 255, 0, 255]),
            hint: Some(StopPosition(LengthUnit::Percentage(50.0))),
          },
          GradientStop::ColorHint {
            color: Color([0, 0, 255, 255]),
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
      center: (0.5, 0.5),
      stops: vec![
        GradientStop::ColorHint {
          color: Color::black(),
          hint: Some(StopPosition(LengthUnit::Percentage(0.0))),
        },
        GradientStop::ColorHint {
          color: Color::black(),
          hint: Some(StopPosition(LengthUnit::Percentage(50.0))),
        },
        GradientStop::ColorHint {
          color: Color::black(),
          hint: Some(StopPosition(LengthUnit::Px(100.0))),
        },
      ],
    };

    let ctx = RenderContext {
      global: &GlobalContext::default(),
      viewport: Viewport::new(200, 100),
      font_size: DEFAULT_FONT_SIZE,
      transform: Affine::identity(),
      style: InheritedStyle::default(),
      draw_debug_border: false,
    };
    let resolved = gradient.resolve_stops_for_radius(ctx.viewport.width as f32, &ctx);

    assert_eq!(resolved.len(), 3);
    assert!((resolved[0].position - 0.0).abs() < 1e-3);
    assert_eq!(resolved[1].position, resolved[2].position);
  }

  #[test]
  fn resolve_stops_equal_positions_distributed_radial() {
    let gradient = RadialGradient {
      shape: RadialShape::Ellipse,
      size: RadialSize::FarthestCorner,
      center: (0.5, 0.5),
      stops: vec![
        GradientStop::ColorHint {
          color: Color::black(),
          hint: Some(StopPosition(LengthUnit::Px(0.0))),
        },
        GradientStop::ColorHint {
          color: Color::black(),
          hint: Some(StopPosition(LengthUnit::Px(0.0))),
        },
        GradientStop::ColorHint {
          color: Color::black(),
          hint: Some(StopPosition(LengthUnit::Px(0.0))),
        },
      ],
    };

    let ctx = RenderContext {
      global: &GlobalContext::default(),
      viewport: Viewport::new(200, 100),
      font_size: DEFAULT_FONT_SIZE,
      transform: Affine::identity(),
      style: InheritedStyle::default(),
      draw_debug_border: false,
    };
    let resolved = gradient.resolve_stops_for_radius(ctx.viewport.width as f32, &ctx);

    assert_eq!(resolved.len(), 3);
    assert!(resolved[0].position >= 0.0);
    assert!(resolved[1].position >= resolved[0].position);
    assert!(resolved[2].position >= resolved[1].position);
  }
}
