use serde::{Deserialize, Serialize};
use swash::text::WordBreakStrength;
use ts_rs::TS;

/// Controls how text should be broken at word boundaries.
///
/// Corresponds to CSS word-break property.
#[derive(Debug, Default, Copy, Clone, Deserialize, Serialize, TS, PartialEq)]
#[serde(rename_all = "kebab-case")]
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

impl From<WordBreak> for WordBreakStrength {
  fn from(value: WordBreak) -> Self {
    match value {
      WordBreak::Normal | WordBreak::BreakWord => WordBreakStrength::Normal,
      WordBreak::BreakAll => WordBreakStrength::BreakAll,
      WordBreak::KeepAll => WordBreakStrength::KeepAll,
    }
  }
}
