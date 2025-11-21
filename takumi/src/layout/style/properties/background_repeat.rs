use cssparser::{Parser, Token, match_ignore_ascii_case};

use crate::layout::style::{FromCss, ParseResult};

/// Per-axis repeat style.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum BackgroundRepeatStyle {
  /// Tile as many times as needed with no extra spacing
  #[default]
  Repeat,
  /// Do not tile on this axis
  NoRepeat,
  /// Distribute leftover space evenly between tiles; edges flush with sides
  Space,
  /// Scale tile so an integer number fits exactly
  Round,
}

/// Combined repeat for X and Y axes.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct BackgroundRepeat(pub BackgroundRepeatStyle, pub BackgroundRepeatStyle);

impl BackgroundRepeat {
  /// Returns a repeat value that tiles on both the X and Y axes.
  pub const fn repeat() -> Self {
    Self(BackgroundRepeatStyle::Repeat, BackgroundRepeatStyle::Repeat)
  }

  /// Returns a repeat value that does not tile on either axis.
  pub const fn no_repeat() -> Self {
    Self(
      BackgroundRepeatStyle::NoRepeat,
      BackgroundRepeatStyle::NoRepeat,
    )
  }

  /// Returns a repeat value that distributes leftover space evenly between tiles; edges flush with sides.
  pub const fn space() -> Self {
    Self(BackgroundRepeatStyle::Space, BackgroundRepeatStyle::Space)
  }

  /// Returns a repeat value that scales tile so an integer number fits exactly.
  pub const fn round() -> Self {
    Self(BackgroundRepeatStyle::Round, BackgroundRepeatStyle::Round)
  }
}

impl<'i> FromCss<'i> for BackgroundRepeat {
  fn from_css(input: &mut Parser<'i, '_>) -> ParseResult<'i, Self> {
    let location = input.current_source_location();
    let first_ident = input.expect_ident_cloned()?;
    let second_ident = input.try_parse(Parser::expect_ident_cloned).ok();

    let parse_axis = |ident: &str| -> Option<BackgroundRepeatStyle> {
      match_ignore_ascii_case! {ident,
        "repeat" => Some(BackgroundRepeatStyle::Repeat),
        "no-repeat" => Some(BackgroundRepeatStyle::NoRepeat),
        "space" => Some(BackgroundRepeatStyle::Space),
        "round" => Some(BackgroundRepeatStyle::Round),
        _ => None,
      }
    };

    match second_ident {
      None => {
        // single keyword forms
        if first_ident.eq_ignore_ascii_case("repeat-x") {
          return Ok(Self(
            BackgroundRepeatStyle::Repeat,
            BackgroundRepeatStyle::NoRepeat,
          ));
        }
        if first_ident.eq_ignore_ascii_case("repeat-y") {
          return Ok(Self(
            BackgroundRepeatStyle::NoRepeat,
            BackgroundRepeatStyle::Repeat,
          ));
        }
        if let Some(axis) = parse_axis(&first_ident) {
          return Ok(Self(axis, axis));
        }
        Err(
          location
            .new_basic_unexpected_token_error(Token::Ident(first_ident.clone()))
            .into(),
        )
      }
      Some(second) => {
        let x = parse_axis(&first_ident).ok_or_else(|| {
          location.new_basic_unexpected_token_error(Token::Ident(first_ident.clone()))
        })?;
        let y = parse_axis(&second)
          .ok_or_else(|| location.new_basic_unexpected_token_error(Token::Ident(second.clone())))?;
        Ok(Self(x, y))
      }
    }
  }
}

/// A list of background-repeat values (layered).
#[derive(Debug, Default, Clone, PartialEq)]
pub struct BackgroundRepeats(pub Vec<BackgroundRepeat>);

impl<'i> FromCss<'i> for BackgroundRepeats {
  fn from_css(input: &mut Parser<'i, '_>) -> ParseResult<'i, Self> {
    let mut values = Vec::new();
    values.push(BackgroundRepeat::from_css(input)?);

    while input.expect_comma().is_ok() {
      values.push(BackgroundRepeat::from_css(input)?);
    }

    Ok(Self(values))
  }
}
