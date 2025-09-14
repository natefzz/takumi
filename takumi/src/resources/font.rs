use std::{
  borrow::Cow,
  collections::{HashMap, HashSet},
  num::NonZeroUsize,
  sync::{Arc, Mutex},
};

use lru::LruCache;

use parley::{
  GenericFamily, Layout, LayoutContext, RangedBuilder, Run,
  fontique::{Blob, FallbackKey, FontInfoOverride, Script},
};
use swash::{
  FontRef,
  scale::{ScaleContext, image::Image, outline::Outline},
};

use crate::layout::style::Color;

/// Represents a resolved glyph that can be either a bitmap image or an outline
#[derive(Clone)]
pub enum ResolvedGlyph {
  /// A bitmap glyph image
  Image(Image),
  /// A vector outline glyph
  Outline(Outline),
}

/// Thread-safe reference-counted glyph for caching
pub type CachedGlyph = Arc<ResolvedGlyph>;

/// Cache key for glyph resolution
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct GlyphCacheKey {
  /// Font identifier
  pub font_id: u32,
  /// Glyph identifier
  pub glyph_id: u16,
  /// Font size (quantized to reduce cache fragmentation)
  pub font_size: u16,
  /// Hash of font variations
  pub variations_hash: u64,
}

/// Combined font scaling and caching context
pub struct FontScaleCache {
  /// Swash scale context
  scale: ScaleContext,
  /// LRU glyph cache for resolved glyphs
  glyph_cache: GlyphCache,
}

/// LRU glyph cache for resolved glyphs
pub struct GlyphCache {
  /// LRU cache with automatic eviction
  cache: LruCache<GlyphCacheKey, CachedGlyph>,
}

impl Default for GlyphCache {
  fn default() -> Self {
    Self {
      cache: LruCache::new(NonZeroUsize::new(1000).unwrap()),
    }
  }
}

impl GlyphCache {
  /// Get a glyph from the cache, updating access order
  pub fn get(&mut self, key: &GlyphCacheKey) -> Option<CachedGlyph> {
    self.cache.get(key).cloned()
  }

  /// Insert a glyph into the cache with automatic LRU eviction
  pub fn insert(&mut self, key: GlyphCacheKey, glyph: ResolvedGlyph) {
    self.cache.put(key, Arc::new(glyph));
  }

  /// Clear all cached glyphs
  pub fn clear(&mut self) {
    self.cache.clear();
  }

  /// Get cache statistics
  pub fn stats(&self) -> (usize, usize) {
    (self.cache.len(), self.cache.cap().get())
  }
}

/// Errors that can occur during font loading and conversion.
#[derive(Debug)]
pub enum FontError {
  /// I/O error occurred while reading the font file
  Io(std::io::Error),
  /// Error occurred during WOFF conversion
  #[cfg(any(feature = "woff", feature = "woff2"))]
  Woff(wuff::WuffErr),
  /// Unsupported Font Format
  UnsupportedFormat,
}

/// Supported font formats for loading and processing
#[derive(Copy, Clone)]
pub enum FontFormat {
  #[cfg(feature = "woff")]
  /// Web Open Font Format (WOFF) - compressed web font format
  Woff,
  #[cfg(feature = "woff2")]
  /// Web Open Font Format 2 (WOFF2) - improved compression web font format
  Woff2,
  /// TrueType Font format - standard desktop font format
  Ttf,
  /// OpenType Font format - extended font format with advanced typography
  Otf,
}

/// Loads and processes font data from raw bytes, optionally using format hint for detection
pub fn load_font<'source>(
  source: &'source [u8],
  format_hint: Option<FontFormat>,
) -> Result<Cow<'source, [u8]>, FontError> {
  let format = if let Some(format) = format_hint {
    format
  } else {
    guess_font_format(source)?
  };

  match format {
    FontFormat::Ttf | FontFormat::Otf => Ok(Cow::Borrowed(source)),
    #[cfg(feature = "woff2")]
    FontFormat::Woff2 => {
      let ttf = wuff::decompress_woff2(source).map_err(FontError::Woff)?;
      Ok(Cow::Owned(ttf))
    }
    #[cfg(feature = "woff")]
    FontFormat::Woff => {
      let ttf = wuff::decompress_woff1(source).map_err(FontError::Woff)?;
      Ok(Cow::Owned(ttf))
    }
  }
}

fn guess_font_format(source: &[u8]) -> Result<FontFormat, FontError> {
  if source.len() < 4 {
    return Err(FontError::UnsupportedFormat);
  }

  match &source[0..4] {
    #[cfg(feature = "woff2")]
    b"wOF2" => Ok(FontFormat::Woff2),
    #[cfg(feature = "woff")]
    b"wOFF" => Ok(FontFormat::Woff),
    [0x00, 0x01, 0x00, 0x00] => Ok(FontFormat::Ttf),
    b"OTTO" => Ok(FontFormat::Otf),
    _ => Err(FontError::UnsupportedFormat),
  }
}

/// A context for managing fonts in the rendering system.
pub struct FontContext {
  layout: Mutex<(parley::FontContext, LayoutContext<Color>)>,
  scale_cache: Mutex<FontScaleCache>,
}

impl Default for FontContext {
  fn default() -> Self {
    Self::new()
  }
}

impl FontContext {
  /// Generate a cache key for glyph resolution
  fn create_cache_key(&self, run: &Run<'_, Color>, glyph_id: u16) -> GlyphCacheKey {
    let font = run.font();
    let synthesis = run.synthesis();
    let variations = synthesis.variations();

    let mut variations_hash = 0u64;
    for variation in variations {
      variations_hash = variations_hash
        .wrapping_mul(31)
        .wrapping_add(variation.tag as u64);
      variations_hash = variations_hash
        .wrapping_mul(31)
        .wrapping_add(variation.value.to_bits() as u64);
    }

    GlyphCacheKey {
      font_id: font.index,
      glyph_id,
      font_size: (run.font_size() * 10.0) as u16, // Quantize to reduce cache fragmentation
      variations_hash,
    }
  }

  /// Get or resolve multiple glyphs using the cache
  /// Returns a HashMap of glyph_id -> CachedGlyph for efficient batch processing
  pub fn get_or_resolve_glyphs(
    &self,
    run: &Run<'_, Color>,
    glyph_ids: impl Iterator<Item = u16> + Clone,
  ) -> HashMap<u16, CachedGlyph> {
    // Collect unique glyph IDs to avoid duplicate work
    let unique_glyph_ids: HashSet<u16> = glyph_ids.collect();

    // Lock both scale and cache together for optimal performance
    let mut scale_cache = self.scale_cache.lock().unwrap();

    // Prepare font info for scaler creation
    let font = run.font();
    let font_ref = FontRef::from_index(font.data.as_ref(), font.index as usize).unwrap();

    let mut result = HashMap::new();

    // Process each unique glyph ID
    for &glyph_id in &unique_glyph_ids {
      let cache_key = self.create_cache_key(run, glyph_id);

      // Try to get from cache first
      if let Some(cached_glyph) = scale_cache.glyph_cache.get(&cache_key) {
        result.insert(glyph_id, cached_glyph);
        continue;
      }

      let mut scaler = scale_cache
        .scale
        .builder(font_ref)
        .size(run.font_size())
        .hint(true)
        .variations(run.synthesis().variations().iter().copied())
        .build();

      let resolved = scaler
        .scale_color_bitmap(glyph_id, swash::scale::StrikeWith::BestFit)
        .map(ResolvedGlyph::Image)
        .or_else(|| {
          scaler
            .scale_color_outline(glyph_id)
            .map(ResolvedGlyph::Outline)
        })
        .or_else(|| scaler.scale_outline(glyph_id).map(ResolvedGlyph::Outline));

      // Cache and return the result if we got one
      if let Some(glyph) = resolved {
        scale_cache.glyph_cache.insert(cache_key, glyph);
        // Get the cached version (now wrapped in Arc)
        if let Some(cached_glyph) = scale_cache.glyph_cache.get(&cache_key) {
          result.insert(glyph_id, cached_glyph);
        }
      }
    }

    result
  }

  /// Get or resolve a single glyph using the cache (backward compatibility)
  pub fn get_or_resolve_glyph(&self, run: &Run<'_, Color>, glyph_id: u16) -> Option<CachedGlyph> {
    self
      .get_or_resolve_glyphs(run, std::iter::once(glyph_id))
      .into_iter()
      .next()
      .map(|(_, glyph)| glyph)
  }

  /// Access the inner context, then return the result of the function
  /// The inner lock will be released after the function is executed
  pub fn create_layout(
    &self,
    text: &str,
    func: impl FnOnce(&mut RangedBuilder<'_, Color>),
  ) -> Layout<Color> {
    let mut lock = self.layout.lock().unwrap();
    let (fcx, lcx) = &mut *lock;

    let mut builder = lcx.ranged_builder(fcx, text, 1.0, true);

    func(&mut builder);

    builder.build(text)
  }

  /// Purge the rasterization cache.
  pub fn purge_cache(&self) {
    let mut lock = self.layout.lock().unwrap();
    lock.0.source_cache.prune(0, true);
  }

  /// Clear the glyph cache
  pub fn purge_glyph_cache(&self) {
    let mut scale_cache = self.scale_cache.lock().unwrap();
    scale_cache.glyph_cache.clear();
  }

  /// Get glyph cache statistics (current_entries, max_entries)
  pub fn glyph_cache_stats(&self) -> (usize, usize) {
    let scale_cache = self.scale_cache.lock().unwrap();
    scale_cache.glyph_cache.stats()
  }

  /// Creates a new font context.
  pub fn new() -> Self {
    Self {
      layout: Mutex::new((parley::FontContext::default(), LayoutContext::default())),
      scale_cache: Mutex::new(FontScaleCache {
        scale: ScaleContext::default(),
        glyph_cache: GlyphCache::default(),
      }),
    }
  }

  /// Loads font into internal font db
  pub fn load_and_store(
    &self,
    source: &[u8],
    info_override: Option<FontInfoOverride<'_>>,
    generic_family: Option<GenericFamily>,
  ) -> Result<(), FontError> {
    let font_data = Blob::new(Arc::new(match load_font(source, None)? {
      Cow::Owned(vec) => vec,
      Cow::Borrowed(slice) => slice.to_vec(),
    }));

    let mut lock = self.layout.lock().unwrap();

    let fonts = lock.0.collection.register_fonts(font_data, info_override);

    for (family, _) in fonts {
      if let Some(generic_family) = generic_family {
        lock
          .0
          .collection
          .append_generic_families(generic_family, std::iter::once(family));
      }

      for (script, _) in Script::all_samples() {
        lock
          .0
          .collection
          .append_fallbacks(FallbackKey::new(*script, None), std::iter::once(family));
      }
    }

    Ok(())
  }
}
