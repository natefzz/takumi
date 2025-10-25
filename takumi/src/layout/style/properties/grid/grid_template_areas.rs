use std::collections::HashMap;

use cssparser::{Parser, ParserInput, Token};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::layout::style::{FromCss, ParseResult};

/// Represents `grid-template-areas` value
///
/// Supports either a 2D matrix of area names (use "." for empty) or a CSS string value
/// like: "a a ." "b b c"
#[derive(Default, Debug, Clone, Deserialize, Serialize, TS, PartialEq)]
#[serde(try_from = "GridTemplateAreasValue")]
#[ts(as = "GridTemplateAreasValue")]
pub struct GridTemplateAreas(pub Vec<Vec<String>>);

/// Serde helper that accepts either a matrix or a CSS string
#[derive(Debug, Clone, Deserialize, Serialize, TS, PartialEq)]
#[serde(untagged)]
pub(crate) enum GridTemplateAreasValue {
  /// A 2D matrix representation (use "." for empty)
  Matrix(Vec<Vec<String>>),
  /// A CSS string representation
  Css(String),
}

impl TryFrom<GridTemplateAreasValue> for GridTemplateAreas {
  type Error = String;

  fn try_from(value: GridTemplateAreasValue) -> Result<Self, Self::Error> {
    match value {
      GridTemplateAreasValue::Matrix(matrix) => {
        // Validate consistent row lengths
        let width = matrix.first().map_or(0, Vec::len);
        if width > 0 && matrix.iter().any(|r| r.len() != width) {
          return Err("Inconsistent row lengths in grid-template-areas matrix".to_string());
        }
        Ok(GridTemplateAreas(matrix))
      }
      GridTemplateAreasValue::Css(css) => {
        let mut input = ParserInput::new(&css);
        let mut parser = Parser::new(&mut input);
        GridTemplateAreas::from_css(&mut parser).map_err(|e| e.to_string())
      }
    }
  }
}

impl From<GridTemplateAreas> for Vec<taffy::GridTemplateArea<String>> {
  fn from(value: GridTemplateAreas) -> Self {
    if value.0.is_empty() {
      return Vec::new();
    }

    let mut bounds: HashMap<&str, (usize, usize, usize, usize)> = HashMap::new();
    for (r, row) in value.0.iter().enumerate() {
      for (c, cell) in row.iter().enumerate() {
        if cell == "." {
          continue;
        }

        let entry = bounds.entry(cell.as_str()).or_insert((r, r, c, c));
        entry.0 = entry.0.min(r);
        entry.1 = entry.1.max(r);
        entry.2 = entry.2.min(c);
        entry.3 = entry.3.max(c);
      }
    }

    let mut areas: Vec<taffy::GridTemplateArea<String>> = Vec::with_capacity(bounds.len());
    for (name, (rmin, rmax, cmin, cmax)) in bounds.into_iter() {
      areas.push(taffy::GridTemplateArea {
        name: name.to_string(),
        row_start: (rmin as u16) + 1,
        row_end: (rmax as u16) + 2,
        column_start: (cmin as u16) + 1,
        column_end: (cmax as u16) + 2,
      });
    }
    areas
  }
}

impl<'i> FromCss<'i> for GridTemplateAreas {
  fn from_css(input: &mut Parser<'i, '_>) -> ParseResult<'i, Self> {
    let mut rows: Vec<Vec<String>> = Vec::new();

    while !input.is_exhausted() {
      let location = input.current_source_location();
      let token = input.next()?;
      match token {
        Token::QuotedString(row) => {
          let row_str: &str = row.as_ref();
          let cols: Vec<String> = row_str
            .split_whitespace()
            .map(ToString::to_string)
            .collect();
          if cols.is_empty() {
            return Err(
              location
                .new_basic_unexpected_token_error(Token::QuotedString(row.clone()))
                .into(),
            );
          }
          rows.push(cols);
        }
        Token::WhiteSpace(_) => continue,
        _ => {
          return Err(
            location
              .new_basic_unexpected_token_error(token.clone())
              .into(),
          );
        }
      }
    }

    // Validate consistent column counts across rows
    if let Some(width) = rows.first().map(Vec::len) {
      if rows.iter().any(|r| r.len() != width) {
        // Create a parse error for inconsistent row lengths
        let location = input.current_source_location();
        return Err(
          location
            .new_basic_unexpected_token_error(Token::Ident("inconsistent-rows".into()))
            .into(),
        );
      }
    }

    Ok(GridTemplateAreas(rows))
  }
}
