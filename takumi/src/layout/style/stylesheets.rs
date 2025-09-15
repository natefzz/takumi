use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use taffy::{Size, prelude::FromLength};
use ts_rs::TS;

use crate::{
  layout::{
    DEFAULT_FONT_SIZE,
    style::{CssValue, properties::*},
  },
  rendering::{RenderContext, SizedShadow},
};

/// Helper macro to define the `Style` struct and `InheritedStyle` struct.
macro_rules! define_style {
  ($( $(#[$attr:meta])? $property:ident: $type:ty = $default_global:expr => $initial_value:expr),* $(,)?) => {
    /// Defines the style of an element.
    #[derive(Debug, Clone, Deserialize, Serialize, TS, Builder)]
    #[serde(default, rename_all = "camelCase")]
    #[ts(export, optional_fields)]
    #[builder(default, setter(into))]
    pub struct Style {
      $(
        #[allow(missing_docs)]
        $(#[$attr])?
        pub $property: CssValue<$type>,
      )*
    }

    impl Default for Style {
      fn default() -> Self {
        Self {
          $( $property: $default_global.into(), )*
        }
      }
    }

    impl Style {
      /// Inherits the style from the parent element.
      pub(crate) fn inherit(&self, parent: &InheritedStyle) -> InheritedStyle {
        InheritedStyle {
          $( $property: self.$property.inherit(&parent.$property, $initial_value), )*
        }
      }
    }

    #[derive(Clone)]
    pub(crate) struct InheritedStyle {
      $( pub $property: $type, )*
    }

    impl Default for InheritedStyle {
      fn default() -> Self {
        Self { $( $property: $initial_value, )* }
      }
    }
  };
}

// property: type = node default value => viewport default value
define_style!(
  // For convenience, we default to border-box
  box_sizing: BoxSizing = CssValue::Inherit => BoxSizing::BorderBox,
  display: Display = Display::Flex => Display::Flex,
  width: LengthUnit = LengthUnit::Auto => LengthUnit::Auto,
  height: LengthUnit = LengthUnit::Auto => LengthUnit::Auto,
  max_width: LengthUnit = LengthUnit::Auto => LengthUnit::Auto,
  max_height: LengthUnit = LengthUnit::Auto => LengthUnit::Auto,
  min_width: LengthUnit = LengthUnit::Auto => LengthUnit::Auto,
  min_height: LengthUnit = LengthUnit::Auto => LengthUnit::Auto,
  aspect_ratio: Option<f32> = None => None,
  padding: Sides<LengthUnit> = Sides::zero() => Sides::zero(),
  padding_top: Option<LengthUnit> = None => None,
  padding_right: Option<LengthUnit> = None => None,
  padding_bottom: Option<LengthUnit> = None => None,
  padding_left: Option<LengthUnit> = None => None,
  margin: Sides<LengthUnit> = Sides::zero() => Sides::zero(),
  margin_top: Option<LengthUnit> = None => None,
  margin_right: Option<LengthUnit> = None => None,
  margin_bottom: Option<LengthUnit> = None => None,
  margin_left: Option<LengthUnit> = None => None,
  inset: Sides<LengthUnit> = Sides::auto() => Sides::auto(),
  top: Option<LengthUnit> = None => None,
  right: Option<LengthUnit> = None => None,
  bottom: Option<LengthUnit> = None => None,
  left: Option<LengthUnit> = None => None,
  flex_direction: FlexDirection = FlexDirection::Row => FlexDirection::Row,
  justify_self: Option<AlignItems> = None => None,
  justify_content: Option<JustifyContent> = None => None,
  align_content: Option<JustifyContent> = None => None,
  justify_items: Option<AlignItems> = None => None,
  align_items: Option<AlignItems> = None => None,
  align_self: Option<AlignItems> = None => None,
  flex_wrap: FlexWrap = FlexWrap::NoWrap => FlexWrap::NoWrap,
  flex_basis: LengthUnit = LengthUnit::Auto => LengthUnit::Auto,
  position: Position = Position::Relative => Position::Relative,
  transform: Option<Transforms> = None => None,
  transform_origin: Option<BackgroundPosition> = None => None,
  mask_image: Option<BackgroundImages> = None => None,
  mask_size: Option<BackgroundSizes> = None => None,
  mask_position: Option<BackgroundPositions> = None => None,
  mask_repeat: Option<BackgroundRepeats> = None => None,
  gap: Gap = Gap::default() => Gap::default(),
  flex_grow: f32 = 0.0 => 0.0,
  flex_shrink: f32 = 1.0 => 1.0,
  border_radius: Sides<LengthUnit> = Sides::zero() => Sides::zero(),
  border_top_left_radius: Option<LengthUnit> = None => None,
  border_top_right_radius: Option<LengthUnit> = None => None,
  border_bottom_right_radius: Option<LengthUnit> = None => None,
  border_bottom_left_radius: Option<LengthUnit> = None => None,
  border_width: Option<Sides<LengthUnit>> = None => None,
  border_top_width: Option<LengthUnit> = None => None,
  border_right_width: Option<LengthUnit> = None => None,
  border_bottom_width: Option<LengthUnit> = None => None,
  border_left_width: Option<LengthUnit> = None => None,
  border: Border = Border::default() => Border::default(),
  object_fit: ObjectFit = CssValue::Inherit => Default::default(),
  object_position: BackgroundPosition = CssValue::Inherit => BackgroundPosition::default(),
  background_image: Option<BackgroundImages> = None => None,
  background_position: Option<BackgroundPositions> = None => None,
  background_size: Option<BackgroundSizes> = None => None,
  background_repeat: Option<BackgroundRepeats> = None => None,
  background_color: Color = Color::transparent() => Color::transparent(),
  box_shadow: Option<BoxShadows> = None => None,
  grid_auto_columns: Option<GridTrackSizes> = None => None,
  grid_auto_rows: Option<GridTrackSizes> = None => None,
  grid_auto_flow: Option<GridAutoFlow> = None => None,
  grid_column: Option<GridLine> = None => None,
  grid_row: Option<GridLine> = None => None,
  grid_template_columns: Option<GridTemplateComponents> = None => None,
  grid_template_rows: Option<GridTemplateComponents> = None => None,
  grid_template_areas: Option<GridTemplateAreas> = None => None,
  text_overflow: TextOverflow = CssValue::Inherit => Default::default(),
  text_transform: TextTransform = CssValue::Inherit => Default::default(),
  font_style: FontStyle = CssValue::Inherit => Default::default(),
  border_color: Option<Color> = CssValue::Inherit => None,
  color: Color = CssValue::Inherit => Color::black(),
  font_size: LengthUnit = CssValue::Inherit => LengthUnit::Px(DEFAULT_FONT_SIZE),
  font_family: Option<FontFamily> = CssValue::Inherit => None,
  line_height: LineHeight = CssValue::Inherit => Default::default(),
  font_weight: FontWeight = CssValue::Inherit => Default::default(),
  font_variation_settings: Option<FontVariationSettings> = CssValue::Inherit => None,
  font_feature_settings: Option<FontFeatureSettings> = CssValue::Inherit => None,
  line_clamp: Option<u32> = CssValue::Inherit => None,
  text_align: TextAlign = CssValue::Inherit => Default::default(),
  text_stroke_width: LengthUnit = CssValue::Inherit => LengthUnit::Px(0.0),
  text_stroke_color: Option<Color> = CssValue::Inherit => None,
  text_stroke: Option<TextStroke> = CssValue::Inherit => None,
  text_shadow: Option<TextShadows> = CssValue::Inherit => None,
  text_decoration: TextDecoration = TextDecoration::default() => TextDecoration::default(),
  text_decoration_line: Option<TextDecorationLines> = CssValue::Inherit => None,
  text_decoration_color: Option<Color> = CssValue::Inherit => None,
  letter_spacing: Option<LengthUnit> = CssValue::Inherit => None,
  word_spacing: Option<LengthUnit> = CssValue::Inherit => None,
  image_rendering: ImageScalingAlgorithm = CssValue::Inherit => Default::default(),
  overflow_wrap: OverflowWrap = CssValue::Inherit => Default::default(),
  word_break: WordBreak = CssValue::Inherit => Default::default(),
);

/// Sized font style with resolved font size and line height.
pub(crate) struct SizedFontStyle<'s> {
  pub parent: &'s InheritedStyle,
  pub font_size: f32,
  pub line_height: parley::LineHeight,
  pub stroke_width: f32,
  pub letter_spacing: Option<f32>,
  pub word_spacing: Option<f32>,
  pub text_shadow: Option<SmallVec<[SizedShadow; 4]>>,
}

impl InheritedStyle {
  #[inline]
  fn convert_template_components(
    components: &Option<GridTemplateComponents>,
    context: &RenderContext,
  ) -> (Vec<taffy::GridTemplateComponent<String>>, Vec<Vec<String>>) {
    let mut track_components: Vec<taffy::GridTemplateComponent<String>> = Vec::new();
    let mut line_name_sets: Vec<Vec<String>> = Vec::new();
    let mut pending_line_names: Vec<String> = Vec::new();

    if let Some(list) = components {
      for comp in list.0.iter() {
        match comp {
          GridTemplateComponent::LineNames(names) => {
            if !names.is_empty() {
              pending_line_names.extend_from_slice(&names[..]);
            }
          }
          GridTemplateComponent::Single(track_size) => {
            // Push names for the line preceding this track
            line_name_sets.push(std::mem::take(&mut pending_line_names));
            // Push the track component
            track_components.push(taffy::GridTemplateComponent::Single(
              track_size.to_min_max(context),
            ));
          }
          GridTemplateComponent::Repeat(repetition, tracks) => {
            // Push names for the line preceding this repeat fragment
            line_name_sets.push(std::mem::take(&mut pending_line_names));

            // Build repetition
            let track_sizes: Vec<taffy::TrackSizingFunction> =
              tracks.iter().map(|t| t.size.to_min_max(context)).collect();

            // Build inner line names: one per line inside the repeat, including a trailing set
            let mut inner_line_names: Vec<Vec<String>> =
              tracks.iter().map(|t| t.names.clone()).collect();
            if let Some(last) = tracks.last() {
              if let Some(end) = &last.end_names {
                inner_line_names.push(end.clone());
              } else {
                inner_line_names.push(Vec::new());
              }
            } else {
              inner_line_names.push(Vec::new());
            }

            track_components.push(taffy::GridTemplateComponent::Repeat(
              taffy::GridTemplateRepetition {
                count: (*repetition).into(),
                tracks: track_sizes,
                line_names: inner_line_names,
              },
            ));
          }
        }
      }
    }

    // Trailing names after the last track
    line_name_sets.push(pending_line_names);

    (track_components, line_name_sets)
  }

  #[inline]
  fn resolve_rect_with_longhands(
    base: Sides<LengthUnit>,
    top: Option<LengthUnit>,
    right: Option<LengthUnit>,
    bottom: Option<LengthUnit>,
    left: Option<LengthUnit>,
  ) -> taffy::Rect<LengthUnit> {
    let mut values = base.0;
    if let Some(v) = top {
      values[0] = v;
    }
    if let Some(v) = right {
      values[1] = v;
    }
    if let Some(v) = bottom {
      values[2] = v;
    }
    if let Some(v) = left {
      values[3] = v;
    }
    taffy::Rect {
      top: values[0],
      right: values[1],
      bottom: values[2],
      left: values[3],
    }
  }

  #[inline]
  fn resolved_padding(&self) -> taffy::Rect<LengthUnit> {
    Self::resolve_rect_with_longhands(
      self.padding,
      self.padding_top,
      self.padding_right,
      self.padding_bottom,
      self.padding_left,
    )
  }

  #[inline]
  fn resolved_margin(&self) -> taffy::Rect<LengthUnit> {
    Self::resolve_rect_with_longhands(
      self.margin,
      self.margin_top,
      self.margin_right,
      self.margin_bottom,
      self.margin_left,
    )
  }

  #[inline]
  fn resolved_inset(&self) -> taffy::Rect<LengthUnit> {
    Self::resolve_rect_with_longhands(self.inset, self.top, self.right, self.bottom, self.left)
  }

  #[inline]
  fn resolved_border_width(&self) -> taffy::Rect<LengthUnit> {
    Self::resolve_rect_with_longhands(
      self
        .border_width
        .or_else(|| self.border.width.map(Into::into))
        .unwrap_or(Sides::zero()),
      self.border_top_width,
      self.border_right_width,
      self.border_bottom_width,
      self.border_left_width,
    )
  }

  #[inline]
  pub(crate) fn resolved_border_radius(&self) -> taffy::Rect<LengthUnit> {
    Self::resolve_rect_with_longhands(
      self.border_radius,
      self.border_top_left_radius,
      self.border_top_right_radius,
      self.border_bottom_right_radius,
      self.border_bottom_left_radius,
    )
  }

  pub fn to_sized_font_style(&'_ self, context: &RenderContext) -> SizedFontStyle<'_> {
    let line_height = self.line_height.into_parley(context);

    let resolved_stroke_width = self
      .text_stroke
      .map(|stroke| stroke.width)
      .unwrap_or(self.text_stroke_width)
      .resolve_to_px(context, context.font_size);

    SizedFontStyle {
      parent: self,
      font_size: context.font_size,
      line_height,
      stroke_width: resolved_stroke_width,
      letter_spacing: self
        .letter_spacing
        .map(|spacing| spacing.resolve_to_px(context, context.font_size) / context.font_size),
      word_spacing: self
        .word_spacing
        .map(|spacing| spacing.resolve_to_px(context, context.font_size) / context.font_size),
      text_shadow: self.text_shadow.as_ref().map(|shadows| {
        shadows
          .0
          .iter()
          .map(|shadow| {
            SizedShadow::from_text_shadow(*shadow, context, Size::from_length(context.font_size))
          })
          .collect()
      }),
    }
  }

  pub fn to_taffy_style(&self, context: &RenderContext) -> taffy::style::Style {
    // Convert grid templates and associated line names
    let (grid_template_columns, grid_template_column_names) =
      Self::convert_template_components(&self.grid_template_columns, context);
    let (grid_template_rows, grid_template_row_names) =
      Self::convert_template_components(&self.grid_template_rows, context);

    taffy::style::Style {
      box_sizing: self.box_sizing.into(),
      size: Size {
        width: self.width.resolve_to_dimension(context),
        height: self.height.resolve_to_dimension(context),
      },
      border: resolve_length_unit_rect_to_length_percentage(context, self.resolved_border_width()),
      padding: resolve_length_unit_rect_to_length_percentage(context, self.resolved_padding()),
      inset: resolve_length_unit_rect_to_length_percentage_auto(context, self.resolved_inset()),
      margin: resolve_length_unit_rect_to_length_percentage_auto(context, self.resolved_margin()),
      display: self.display.into(),
      flex_direction: self.flex_direction.into(),
      position: self.position.into(),
      justify_content: self.justify_content.map(Into::into),
      align_content: self.align_content.map(Into::into),
      justify_items: self.justify_items.map(Into::into),
      flex_grow: self.flex_grow,
      align_items: self.align_items.map(Into::into),
      gap: self.gap.resolve_to_size(context),
      flex_basis: self.flex_basis.resolve_to_dimension(context),
      flex_shrink: self.flex_shrink,
      flex_wrap: self.flex_wrap.into(),
      min_size: Size {
        width: self.min_width.resolve_to_dimension(context),
        height: self.min_height.resolve_to_dimension(context),
      },
      max_size: Size {
        width: self.max_width.resolve_to_dimension(context),
        height: self.max_height.resolve_to_dimension(context),
      },
      grid_auto_columns: self.grid_auto_columns.as_ref().map_or_else(Vec::new, |v| {
        v.0.iter().map(|s| s.to_min_max(context)).collect()
      }),
      grid_auto_rows: self.grid_auto_rows.as_ref().map_or_else(Vec::new, |v| {
        v.0.iter().map(|s| s.to_min_max(context)).collect()
      }),
      grid_auto_flow: self.grid_auto_flow.unwrap_or_default().into(),
      grid_column: self
        .grid_column
        .as_ref()
        .map_or_else(Default::default, |line| line.clone().into()),
      grid_row: self
        .grid_row
        .as_ref()
        .map_or_else(Default::default, |line| line.clone().into()),
      grid_template_columns,
      grid_template_rows,
      grid_template_column_names,
      grid_template_row_names,
      grid_template_areas: self
        .grid_template_areas
        .as_ref()
        .cloned()
        .unwrap_or_default()
        .into(),
      aspect_ratio: self.aspect_ratio,
      align_self: self.align_self.map(Into::into),
      justify_self: self.justify_self.map(Into::into),
      ..Default::default()
    }
  }
}
