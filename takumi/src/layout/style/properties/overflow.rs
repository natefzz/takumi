use std::ops::Deref;

use cssparser::{Parser, match_ignore_ascii_case};
use serde::{Deserialize, Serialize};
use taffy::{Layout, Point};
use ts_rs::TS;

use crate::{
  layout::style::{Affine, FromCss, ParseResult, SpacePair, tw::TailwindPropertyParser},
  rendering::CanvasConstrain,
};

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

impl TailwindPropertyParser for Overflow {
  fn parse_tw(token: &str) -> Option<Self> {
    match_ignore_ascii_case! {token,
      "visible" => Some(Overflow::Visible),
      "hidden" => Some(Overflow::Hidden),
      _ => None,
    }
  }
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
#[serde(transparent)]
pub struct Overflows(pub SpacePair<Overflow>);

impl From<Overflows> for Point<taffy::Overflow> {
  fn from(val: Overflows) -> Self {
    Point {
      x: val.x.into(),
      y: val.y.into(),
    }
  }
}

impl Default for Overflows {
  fn default() -> Self {
    Self(SpacePair::from_single(Overflow::Visible))
  }
}

impl Deref for Overflows {
  type Target = SpacePair<Overflow>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl Overflows {
  #[inline]
  pub(crate) fn should_clip_content(&self) -> bool {
    *self != Overflows(SpacePair::from_single(Overflow::Visible))
  }

  pub(crate) fn create_constrain(
    &self,
    layout: Layout,
    transform: Affine,
  ) -> Option<CanvasConstrain> {
    let clip_x = self.x != Overflow::Visible;
    let clip_y = self.y != Overflow::Visible;

    if !self.should_clip_content()
      || (clip_x && layout.content_box_width() < f32::EPSILON)
      || (clip_y && layout.content_box_height() < f32::EPSILON)
    {
      return None;
    }

    let from = Point {
      x: if clip_x {
        layout.padding.left + layout.border.left
      } else {
        f32::MIN
      },
      y: if clip_y {
        layout.padding.top + layout.border.top
      } else {
        f32::MIN
      },
    };
    let to = Point {
      x: if clip_x {
        from.x + layout.content_box_width()
      } else {
        f32::MAX
      },
      y: if clip_y {
        from.y + layout.content_box_height()
      } else {
        f32::MAX
      },
    };

    Some(CanvasConstrain {
      from,
      to,
      inverse_transform: transform.invert()?,
      mask: None,
    })
  }
}

#[cfg(test)]
mod tests {
  use serde_json;

  use super::*;

  #[test]
  fn test_overflow_deserialize() {
    // Test deserialization from string (single value)
    let overflow_json = r#""hidden""#;
    let overflow: Overflows = serde_json::from_str(overflow_json).unwrap();
    assert_eq!(overflow.x, Overflow::Hidden);
    assert_eq!(overflow.y, Overflow::Hidden);

    // Test deserialization from object (pair of values)
    let overflow_json = r#"{"x": "visible", "y": "hidden"}"#;
    let overflow: Overflows = serde_json::from_str(overflow_json).unwrap();
    assert_eq!(overflow.x, Overflow::Visible);
    assert_eq!(overflow.y, Overflow::Hidden);
  }
}
