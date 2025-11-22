use cssparser::Parser;

use crate::layout::style::{FromCss, LengthUnit, ParseResult, tw::TailwindPropertyParser};

/// Parsed `background-size` for one layer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BackgroundSize {
  /// Scale the image to cover the container (may crop).
  Cover,
  /// Scale the image to be fully contained within the container.
  Contain,
  /// Explicit width and height values.
  Explicit {
    /// Width value for the background image.
    width: LengthUnit,
    /// Height value for the background image.
    height: LengthUnit,
  },
}

impl TailwindPropertyParser for BackgroundSize {
  fn parse_tw(token: &str) -> Option<Self> {
    match token {
      "cover" => Some(BackgroundSize::Cover),
      "contain" => Some(BackgroundSize::Contain),
      _ => None,
    }
  }
}

impl Default for BackgroundSize {
  fn default() -> Self {
    BackgroundSize::Explicit {
      width: LengthUnit::Auto,
      height: LengthUnit::Auto,
    }
  }
}

impl<'i> FromCss<'i> for BackgroundSize {
  fn from_css(input: &mut Parser<'i, '_>) -> ParseResult<'i, Self> {
    if input
      .try_parse(|input| input.expect_ident_matching("cover"))
      .is_ok()
    {
      return Ok(BackgroundSize::Cover);
    }

    if input
      .try_parse(|input| input.expect_ident_matching("contain"))
      .is_ok()
    {
      return Ok(BackgroundSize::Contain);
    }

    let first = LengthUnit::from_css(input)?;
    let Ok(second) = input.try_parse(LengthUnit::from_css) else {
      return Ok(BackgroundSize::Explicit {
        width: first,
        height: first,
      });
    };

    Ok(BackgroundSize::Explicit {
      width: first,
      height: second,
    })
  }
}

/// A list of `background-size` values (one per layer).
#[derive(Debug, Default, Clone, PartialEq)]
pub struct BackgroundSizes(pub Vec<BackgroundSize>);

impl<'i> FromCss<'i> for BackgroundSizes {
  fn from_css(input: &mut Parser<'i, '_>) -> ParseResult<'i, Self> {
    let mut values = Vec::new();
    values.push(BackgroundSize::from_css(input)?);

    while input.expect_comma().is_ok() {
      values.push(BackgroundSize::from_css(input)?);
    }

    Ok(Self(values))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parses_cover_keyword() {
    assert_eq!(BackgroundSize::from_str("cover"), Ok(BackgroundSize::Cover));
  }

  #[test]
  fn parses_contain_keyword() {
    assert_eq!(
      BackgroundSize::from_str("contain"),
      Ok(BackgroundSize::Contain)
    );
  }

  #[test]
  fn parses_single_percentage_value_as_both_dimensions() {
    assert_eq!(
      BackgroundSize::from_str("50%\t"),
      Ok(BackgroundSize::Explicit {
        width: LengthUnit::Percentage(50.0),
        height: LengthUnit::Percentage(50.0),
      })
    );
  }

  #[test]
  fn parses_single_auto_value_as_both_dimensions() {
    assert_eq!(
      BackgroundSize::from_str("auto"),
      Ok(BackgroundSize::Explicit {
        width: LengthUnit::Auto,
        height: LengthUnit::Auto,
      })
    );
  }

  #[test]
  fn parses_two_values_mixed_units() {
    assert_eq!(
      BackgroundSize::from_str("100px auto"),
      Ok(BackgroundSize::Explicit {
        width: LengthUnit::Px(100.0),
        height: LengthUnit::Auto,
      })
    );
  }

  #[test]
  fn errors_on_unknown_identifier() {
    assert!(BackgroundSize::from_str("bogus").is_err());
  }

  #[test]
  fn parses_multiple_layers_with_keywords_and_values() {
    assert_eq!(
      BackgroundSizes::from_str("cover, 50% auto"),
      Ok(BackgroundSizes(vec![
        BackgroundSize::Cover,
        BackgroundSize::Explicit {
          width: LengthUnit::Percentage(50.0),
          height: LengthUnit::Auto,
        }
      ]))
    );
  }

  #[test]
  fn parses_multiple_layers_with_single_value_duplication() {
    assert_eq!(
      BackgroundSizes::from_str("25%, contain"),
      Ok(BackgroundSizes(vec![
        BackgroundSize::Explicit {
          width: LengthUnit::Percentage(25.0),
          height: LengthUnit::Percentage(25.0),
        },
        BackgroundSize::Contain
      ]))
    );
  }

  #[test]
  fn errors_on_invalid_first_layer() {
    assert!(BackgroundSizes::from_str("nope").is_err());
  }
}
