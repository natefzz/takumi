use smallvec::SmallVec;

use super::{Color, GradientStop, LengthUnit, ResolvedGradientStop};
use crate::rendering::RenderContext;

/// Interpolates between two colors in RGBA space, if t is 0.0 or 1.0, returns the first or second color.
pub(crate) fn interpolate_rgba(c1: Color, c2: Color, t: f32) -> Color {
  if t <= f32::EPSILON {
    return c1;
  }
  if t >= 1.0 - f32::EPSILON {
    return c2;
  }

  let mut out = [0u8; 4];

  for (i, value) in out.iter_mut().enumerate() {
    *value = (c1.0[i] as f32 * (1.0 - t) + c2.0[i] as f32 * t).round() as u8;
  }

  Color(out)
}

/// Returns the color for a pixel-space position along the resolved stops.
pub(crate) fn color_from_stops(position: f32, resolved_stops: &[ResolvedGradientStop]) -> Color {
  // Find the two stops that bracket the current position.
  // We want the last stop with position <= current position.
  let left_index = resolved_stops
    .iter()
    .rposition(|stop| stop.position <= position)
    .unwrap_or(0);

  let right_index = resolved_stops
    .iter()
    .enumerate()
    .position(|(i, stop)| i > left_index && stop.position >= position)
    .unwrap_or(resolved_stops.len() - 1);

  if left_index == right_index {
    // if the left and right indices are the same, we should return a hard stop
    resolved_stops[left_index].color
  } else {
    let left_stop = &resolved_stops[left_index];
    let right_stop = &resolved_stops[right_index];

    let denom = right_stop.position - left_stop.position;
    let interpolation_position = if denom.abs() < f32::EPSILON {
      0.0
    } else {
      ((position - left_stop.position) / denom).clamp(0.0, 1.0)
    };

    interpolate_rgba(left_stop.color, right_stop.color, interpolation_position)
  }
}

/// Resolve a length value used in a gradient stop to an absolute pixel position along an axis.
pub(crate) fn stop_length_to_axis_px(
  length: LengthUnit,
  axis_size_px: f32,
  context: &RenderContext,
) -> f32 {
  length.resolve_to_px(context, axis_size_px).max(0.0)
}

/// Resolves gradient steps into color stops with positions expressed in pixels along an axis.
/// The `treat_equal_as_non_increasing` flag controls whether equal neighbor positions should be
/// treated as non-increasing (and thus distributed). Linear gradients allow equal positions to
/// create hard color jumps, while radial gradients previously distributed equals.
pub(crate) fn resolve_stops_along_axis(
  stops: &[GradientStop],
  axis_size_px: f32,
  context: &RenderContext,
) -> SmallVec<[ResolvedGradientStop; 4]> {
  let mut resolved: SmallVec<[ResolvedGradientStop; 4]> = SmallVec::new();
  for (i, step) in stops.iter().enumerate() {
    match step {
      GradientStop::ColorHint { color, hint } => {
        let position = hint
          .map(|sp| stop_length_to_axis_px(sp.0, axis_size_px, context))
          .unwrap_or(-1.0);

        resolved.push(ResolvedGradientStop {
          color: *color,
          position,
        });
      }
      GradientStop::Hint(hint) => {
        let before_color = resolved
          .get(i - 1)
          .expect("Gradient hint found without a preceding color stop. Each hint must follow a color stop.")
          .color;

        let after_color = stops
          .get(i + 1)
          .and_then(|stop| match stop {
            GradientStop::ColorHint { color, hint: _ } => Some(*color),
            GradientStop::Hint(_) => None,
          })
          .expect("Gradient hint found without a following color stop. Each hint must be followed by a color stop.");

        let interpolated_color = interpolate_rgba(before_color, after_color, 0.5);

        resolved.push(ResolvedGradientStop {
          color: interpolated_color,
          position: stop_length_to_axis_px(hint.0, axis_size_px, context),
        });
      }
    }
  }

  // If there are no color stops, return an empty vector
  if resolved.is_empty() {
    return resolved;
  }

  if resolved[0].position == -1.0 {
    resolved[0].position = 0.0;
  }

  // If there is only one stop, set its position to 0.0
  if resolved.len() == 1 {
    return resolved;
  }

  // Distribute unspecified or non-increasing positions in pixel domain
  let mut i = 0usize;
  while i < resolved.len() {
    // if the position is defined, skip it
    if resolved[i].position >= 0.0 {
      i += 1;
      continue;
    }

    let last_defined_position = if i == 0 {
      0.0
    } else {
      resolved
        .get(i - 1)
        .map(|s| s.position)
        .unwrap_or(0.0)
        .max(0.0)
    };

    // try to find next defined position, if not found, use the axis size
    let next_index = match resolved.iter().skip(i + 1).position(|s| s.position >= 0.0) {
      Some(rel_idx) => i + 1 + rel_idx,
      None => resolved.len() - 1,
    };

    let next_position = if resolved[next_index].position < 0.0 {
      axis_size_px
    } else {
      resolved[next_index].position
    };

    // number of segments between last defined and next position includes the next slot
    let segments_count = (next_index - i + 1) as f32;
    let step_for_each_segment = (next_position - last_defined_position) / segments_count;

    // distribute the step evenly between the stops, ensuring the last stop reaches next_position
    for (stop_index, stop) in resolved
      .iter_mut()
      .enumerate()
      .skip(i)
      .take(next_index - i + 1)
    {
      stop.position = last_defined_position + step_for_each_segment * ((stop_index - i + 1) as f32);
    }

    i = next_index + 1;
  }

  resolved
}

#[cfg(test)]
mod tests {

  use crate::{
    GlobalContext,
    layout::{
      DEFAULT_FONT_SIZE, Viewport,
      style::{Affine, InheritedStyle, StopPosition},
    },
  };

  use super::*;

  #[test]
  fn test_resolve_stops_along_axis() {
    let stops = vec![
      GradientStop::ColorHint {
        color: Color([255, 0, 0, 255]),
        hint: Some(StopPosition(LengthUnit::Px(10.0))),
      },
      GradientStop::ColorHint {
        color: Color([0, 255, 0, 255]),
        hint: Some(StopPosition(LengthUnit::Px(20.0))),
      },
      GradientStop::ColorHint {
        color: Color([0, 0, 255, 255]),
        hint: Some(StopPosition(LengthUnit::Percentage(30.0))),
      },
    ];

    let ctx = RenderContext {
      global: &GlobalContext::default(),
      viewport: Viewport::new(40, 40),
      font_size: DEFAULT_FONT_SIZE,
      transform: Affine::identity(),
      style: InheritedStyle::default(),
    };

    let resolved = resolve_stops_along_axis(&stops, ctx.viewport.width as f32, &ctx);

    assert_eq!(
      resolved[0],
      ResolvedGradientStop {
        color: Color([255, 0, 0, 255]),
        position: 10.0,
      },
    );

    assert_eq!(
      resolved[1],
      ResolvedGradientStop {
        color: Color([0, 255, 0, 255]),
        position: 20.0,
      },
    );

    assert_eq!(
      resolved[2],
      ResolvedGradientStop {
        color: Color([0, 0, 255, 255]),
        position: ctx.viewport.width as f32 * 0.3,
      },
    );
  }

  #[test]
  fn test_distribute_evenly_between_positions() {
    let stops = vec![
      GradientStop::ColorHint {
        color: Color([255, 0, 0, 255]),
        hint: None,
      },
      GradientStop::ColorHint {
        color: Color([0, 255, 0, 255]),
        hint: None,
      },
      GradientStop::ColorHint {
        color: Color([0, 0, 255, 255]),
        hint: None,
      },
    ];

    let ctx = RenderContext {
      global: &GlobalContext::default(),
      viewport: Viewport::new(40, 40),
      font_size: DEFAULT_FONT_SIZE,
      transform: Affine::identity(),
      style: InheritedStyle::default(),
    };

    let resolved = resolve_stops_along_axis(&stops, ctx.viewport.width as f32, &ctx);

    assert_eq!(
      resolved[0],
      ResolvedGradientStop {
        color: Color([255, 0, 0, 255]),
        position: 0.0,
      },
    );

    assert_eq!(
      resolved[1],
      ResolvedGradientStop {
        color: Color([0, 255, 0, 255]),
        position: ctx.viewport.width as f32 / 2.0,
      },
    );

    assert_eq!(
      resolved[2],
      ResolvedGradientStop {
        color: Color([0, 0, 255, 255]),
        position: ctx.viewport.width as f32,
      },
    );
  }

  #[test]
  fn test_hint_only() {
    let stops = vec![
      GradientStop::ColorHint {
        color: Color([255, 0, 0, 255]),
        hint: None,
      },
      GradientStop::Hint(StopPosition(LengthUnit::Percentage(10.0))),
      GradientStop::ColorHint {
        color: Color([0, 0, 255, 255]),
        hint: None,
      },
    ];

    let ctx = RenderContext {
      global: &GlobalContext::default(),
      viewport: Viewport::new(100, 40),
      font_size: DEFAULT_FONT_SIZE,
      transform: Affine::identity(),
      style: InheritedStyle::default(),
    };

    let resolved = resolve_stops_along_axis(&stops, ctx.viewport.width as f32, &ctx);

    assert_eq!(
      resolved[0],
      ResolvedGradientStop {
        color: Color([255, 0, 0, 255]),
        position: 0.0,
      },
    );

    // the mid color between red and blue should be at 10%
    assert_eq!(
      resolved[1],
      ResolvedGradientStop {
        color: interpolate_rgba(Color([255, 0, 0, 255]), Color([0, 0, 255, 255]), 0.5),
        position: ctx.viewport.width as f32 * 0.1,
      },
    );

    assert_eq!(
      resolved[2],
      ResolvedGradientStop {
        color: Color([0, 0, 255, 255]),
        position: ctx.viewport.width as f32,
      },
    );
  }
}
