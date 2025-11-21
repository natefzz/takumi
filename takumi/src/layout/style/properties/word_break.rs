use cssparser::{Parser, Token, match_ignore_ascii_case};
use swash::text::WordBreakStrength;

use crate::layout::style::{FromCss, ParseResult};

/// Controls how text should be broken at word boundaries.
///
/// Corresponds to CSS word-break property.
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub enum WordBreak {
  /// Normal line breaking behavior—lines may break according to language rules.
  #[default]
  Normal,
  /// Break words at arbitrary points to prevent overflow.
  BreakAll,
  /// Prevents word breaks within words. Useful for languages like Japanese.
  KeepAll,
  /// Allow breaking within long words if necessary to prevent overflow.
  BreakWord,
}

impl<'i> FromCss<'i> for WordBreak {
  fn from_css(input: &mut Parser<'i, '_>) -> ParseResult<'i, Self> {
    let location = input.current_source_location();
    let ident = input.expect_ident()?;

    match_ignore_ascii_case! {&ident,
      "normal" => Ok(WordBreak::Normal),
      "break-all" => Ok(WordBreak::BreakAll),
      "keep-all" => Ok(WordBreak::KeepAll),
      "break-word" => Ok(WordBreak::BreakWord),
      _ => Err(location.new_unexpected_token_error(Token::Ident(ident.clone()))),
    }
  }
}

impl From<WordBreak> for WordBreakStrength {
  fn from(value: WordBreak) -> Self {
    match value {
      WordBreak::Normal | WordBreak::BreakWord => WordBreakStrength::Normal,
      WordBreak::BreakAll => WordBreakStrength::BreakAll,
      WordBreak::KeepAll => WordBreakStrength::KeepAll,
    }
  }
}
