use cssparser::Parser;
use parley::FontVariation;
use smallvec::SmallVec;

use crate::layout::style::{FromCss, ParseResult};

/// Controls variable font axis values via CSS font-variation-settings property.
///
/// This allows fine-grained control over variable font characteristics like weight,
/// width, slant, and other custom axes defined in the font.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FontVariationSettings(pub SmallVec<[FontVariation; 4]>);

impl<'i> FromCss<'i> for FontVariationSettings {
  fn from_css(input: &mut Parser<'i, '_>) -> ParseResult<'i, Self> {
    Ok(FontVariationSettings(
      FontVariation::parse_list(input.current_line()).collect(),
    ))
  }
}
