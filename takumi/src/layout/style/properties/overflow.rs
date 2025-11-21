use cssparser::{Parser, match_ignore_ascii_case};

use crate::layout::style::{FromCss, ParseResult, tw::TailwindPropertyParser};

/// How children overflowing their container should affect layout
#[derive(Debug, Clone, Copy, PartialEq, Default)]
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
