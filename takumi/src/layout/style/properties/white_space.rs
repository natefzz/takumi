use cssparser::{Parser, Token, match_ignore_ascii_case};

use crate::layout::style::{
  FromCss, ParseResult, TextWrapMode, WhiteSpaceCollapse, tw::TailwindPropertyParser,
};

/// Controls how whitespace should be handled.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct WhiteSpace {
  /// Controls whether text should be wrapped.
  pub text_wrap_mode: TextWrapMode,
  /// Controls how whitespace should be collapsed.
  pub white_space_collapse: WhiteSpaceCollapse,
}

impl TailwindPropertyParser for WhiteSpace {
  fn parse_tw(token: &str) -> Option<Self> {
    match_ignore_ascii_case! {token,
      "normal" => Some(WhiteSpace::normal()),
      "nowrap" => Some(WhiteSpace::no_wrap()),
      "pre" => Some(WhiteSpace::pre()),
      "pre-wrap" => Some(WhiteSpace::pre_wrap()),
      "pre-line" => Some(WhiteSpace::pre_line()),
      _ => None,
    }
  }
}

impl WhiteSpace {
  /// Creates a `WhiteSpace` instance with `nowrap` behavior.
  pub const fn no_wrap() -> Self {
    Self {
      text_wrap_mode: TextWrapMode::NoWrap,
      white_space_collapse: WhiteSpaceCollapse::Collapse,
    }
  }

  /// Creates a `WhiteSpace` instance with `normal` behavior.
  pub const fn normal() -> Self {
    Self {
      text_wrap_mode: TextWrapMode::Wrap,
      white_space_collapse: WhiteSpaceCollapse::Collapse,
    }
  }

  /// Creates a `WhiteSpace` instance with `pre` behavior.
  pub const fn pre() -> Self {
    Self {
      text_wrap_mode: TextWrapMode::NoWrap,
      white_space_collapse: WhiteSpaceCollapse::Preserve,
    }
  }

  /// Creates a `WhiteSpace` instance with `pre-wrap` behavior.
  pub const fn pre_wrap() -> Self {
    Self {
      text_wrap_mode: TextWrapMode::Wrap,
      white_space_collapse: WhiteSpaceCollapse::Preserve,
    }
  }

  /// Creates a `WhiteSpace` instance with `pre-line` behavior.
  pub const fn pre_line() -> Self {
    Self {
      text_wrap_mode: TextWrapMode::Wrap,
      white_space_collapse: WhiteSpaceCollapse::PreserveBreaks,
    }
  }
}

impl<'i> FromCss<'i> for WhiteSpace {
  fn from_css(input: &mut Parser<'i, '_>) -> ParseResult<'i, Self> {
    // Try parsing as a keyword first
    if let Ok(ident) = input.try_parse(Parser::expect_ident_cloned) {
      return match_ignore_ascii_case! {&ident,
        "normal" => Ok(WhiteSpace::normal()),
        "pre" => Ok(WhiteSpace::pre()),
        "pre-wrap" => Ok(WhiteSpace::pre_wrap()),
        "pre-line" => Ok(WhiteSpace::pre_line()),
        _ => {
          let token = Token::Ident(ident.clone());
          Err(input.new_basic_unexpected_token_error(token).into())
        }
      };
    }

    // Otherwise parse individual components
    let mut text_wrap_mode = None;
    let mut white_space_collapse = None;

    while !input.is_exhausted() {
      if let Ok(value) = input.try_parse(TextWrapMode::from_css) {
        text_wrap_mode = Some(value);
        continue;
      }

      if let Ok(value) = input.try_parse(WhiteSpaceCollapse::from_css) {
        white_space_collapse = Some(value);
        continue;
      }

      return Err(input.new_error_for_next_token());
    }

    Ok(WhiteSpace {
      text_wrap_mode: text_wrap_mode.unwrap_or_default(),
      white_space_collapse: white_space_collapse.unwrap_or_default(),
    })
  }
}
