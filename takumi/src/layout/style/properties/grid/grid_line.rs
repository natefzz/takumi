use cssparser::Parser;

use crate::layout::style::{FromCss, ParseResult};

use super::GridPlacement;

/// Represents a grid line placement with serde support
#[derive(Debug, Clone, Default, PartialEq)]
pub struct GridLine {
  /// The start line placement
  pub start: Option<GridPlacement>,
  /// The end line placement
  pub end: Option<GridPlacement>,
}

impl From<GridLine> for taffy::Line<taffy::GridPlacement> {
  fn from(line: GridLine) -> Self {
    Self {
      start: line.start.unwrap_or_default().into(),
      end: line.end.unwrap_or_default().into(),
    }
  }
}

impl<'i> FromCss<'i> for GridLine {
  fn from_css(input: &mut Parser<'i, '_>) -> ParseResult<'i, Self> {
    // First placement is required
    let first = GridPlacement::from_css(input).ok();

    // Optional delimiter '/'
    let second = if input.try_parse(|i| i.expect_delim('/')).is_ok() {
      GridPlacement::from_css(input).ok()
    } else {
      None
    };

    if first.is_none() && second.is_none() {
      return Err(input.new_error_for_next_token());
    }

    Ok(GridLine {
      start: first,
      end: second,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_parse_line() {
    assert_eq!(
      GridLine::from_str("span 2 / 3"),
      Ok(GridLine {
        start: Some(GridPlacement::span(2)),
        end: Some(GridPlacement::Line(3)),
      })
    );
  }
}
