use cssparser::Parser;
use parley::style::FontStyle as ParleyFontStyle;

use crate::layout::style::{FromCss, ParseResult};

/// Controls the slant (italic/oblique) of text rendering.
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct FontStyle(ParleyFontStyle);

impl<'i> FromCss<'i> for FontStyle {
  fn from_css(input: &mut Parser<'i, '_>) -> ParseResult<'i, Self> {
    ParleyFontStyle::parse(input.current_line())
      .map(FontStyle)
      .ok_or_else(|| input.new_error_for_next_token())
  }
}

impl FontStyle {
  /// The normal font style.
  pub const fn normal() -> Self {
    Self(ParleyFontStyle::Normal)
  }

  /// The italic font style.
  pub const fn italic() -> Self {
    Self(ParleyFontStyle::Italic)
  }

  /// The oblique font style with a given angle.
  pub const fn oblique(angle: f32) -> Self {
    Self(ParleyFontStyle::Oblique(Some(angle)))
  }
}

impl From<FontStyle> for ParleyFontStyle {
  fn from(value: FontStyle) -> Self {
    value.0
  }
}
