//! kami-text: SDF (Signed Distance Field) text rendering for wgpu.
//!
//! GPU-accelerated text with pixel-perfect quality at any zoom level.
//! Generates SDF glyph atlas at runtime from font metrics.

use ab_glyph::{Font, GlyphId, ScaleFont};
use bytemuck::{Pod, Zeroable};
use glam::{Vec2, Vec4};
use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use unicode_segmentation::UnicodeSegmentation;

pub const DEFAULT_LATIN_FONT_BYTES: &[u8] = include_bytes!("../fonts/Poppins-Regular.ttf");
pub const DEFAULT_CJK_FONT_BYTES: &[u8] = include_bytes!("../fonts/NotoSansJP-Regular.otf");

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FontSlot {
    Latin,
    Cjk,
    Emoji,
}

impl FontSlot {
    fn as_u8(self) -> u8 {
        match self {
            Self::Latin => 0,
            Self::Cjk => 1,
            Self::Emoji => 2,
        }
    }

    fn from_u8(value: u8) -> Self {
        match value {
            1 => Self::Cjk,
            2 => Self::Emoji,
            _ => Self::Latin,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GlyphKey {
    pub font_slot: u8,
    pub glyph_id: u16,
}

impl GlyphKey {
    fn new(slot: FontSlot, glyph_id: u16) -> Self {
        Self {
            font_slot: slot.as_u8(),
            glyph_id,
        }
    }

    fn slot(self) -> FontSlot {
        FontSlot::from_u8(self.font_slot)
    }
}

#[derive(Debug, Clone)]
struct ShapedGlyph {
    key: GlyphKey,
    x_advance: f32,
    x_offset: f32,
    y_offset: f32,
}

pub struct FontManager {
    latin: ab_glyph::FontArc,
    cjk: ab_glyph::FontArc,
    emoji: Option<ab_glyph::FontArc>,
    latin_face: rustybuzz::Face<'static>,
    cjk_face: rustybuzz::Face<'static>,
    emoji_face: Option<rustybuzz::Face<'static>>,
}

impl FontManager {
    pub fn new_default() -> Result<Self, ab_glyph::InvalidFont> {
        Self::new_with_options(EmojiFontOptions::default())
    }

    pub fn new_with_options(options: EmojiFontOptions) -> Result<Self, ab_glyph::InvalidFont> {
        let emoji_bytes = resolve_emoji_font_bytes(&options);
        let emoji = emoji_bytes.and_then(|bytes| ab_glyph::FontArc::try_from_slice(bytes).ok());
        let emoji_face = emoji_bytes.and_then(|bytes| rustybuzz::Face::from_slice(bytes, 0));
        Ok(Self {
            latin: ab_glyph::FontArc::try_from_slice(DEFAULT_LATIN_FONT_BYTES)?,
            cjk: ab_glyph::FontArc::try_from_slice(DEFAULT_CJK_FONT_BYTES)?,
            emoji,
            latin_face: rustybuzz::Face::from_slice(DEFAULT_LATIN_FONT_BYTES, 0)
                .expect("bundled Poppins font should be valid"),
            cjk_face: rustybuzz::Face::from_slice(DEFAULT_CJK_FONT_BYTES, 0)
                .expect("bundled Noto Sans JP font should be valid"),
            emoji_face,
        })
    }

    pub fn font_for_char(&self, ch: char) -> &ab_glyph::FontArc {
        self.font_for_slot(self.slot_for_char(ch))
    }

    fn font_for_slot(&self, slot: FontSlot) -> &ab_glyph::FontArc {
        match slot {
            FontSlot::Latin => &self.latin,
            FontSlot::Cjk => &self.cjk,
            FontSlot::Emoji => self.emoji.as_ref().unwrap_or(&self.cjk),
        }
    }

    fn face_for_slot(&self, slot: FontSlot) -> &rustybuzz::Face<'static> {
        match slot {
            FontSlot::Latin => &self.latin_face,
            FontSlot::Cjk => &self.cjk_face,
            FontSlot::Emoji => self.emoji_face.as_ref().unwrap_or(&self.cjk_face),
        }
    }

    fn slot_for_char(&self, ch: char) -> FontSlot {
        if prefers_emoji_font(ch) && self.emoji.is_some() {
            FontSlot::Emoji
        } else if prefers_cjk_font(ch) {
            FontSlot::Cjk
        } else {
            FontSlot::Latin
        }
    }

    pub fn fallback_font(&self) -> &ab_glyph::FontArc {
        &self.latin
    }

    pub fn has_emoji_font(&self) -> bool {
        self.emoji.is_some() && self.emoji_face.is_some()
    }

    pub fn build_atlas(
        &self,
        font_size: f32,
        charset: &str,
    ) -> Result<FontAtlas, ab_glyph::InvalidFont> {
        let inventory = self.collect_glyph_inventory(font_size, charset);
        self.build_atlas_from_inventory(font_size, &inventory)
    }

    fn collect_glyph_inventory(&self, font_size: f32, text: &str) -> GlyphInventory {
        let mut keys = BTreeSet::new();
        let mut char_map = HashMap::new();

        for ch in normalized_charset(text) {
            let key = self.direct_glyph_key(ch);
            keys.insert(key);
            char_map.entry(ch).or_insert(key);
        }

        for run in segment_text_runs(text) {
            if run.text == "\n" || run.text.is_empty() {
                continue;
            }
            for glyph in self.shape_run(font_size, &run) {
                keys.insert(glyph.key);
            }
        }

        GlyphInventory { keys, char_map }
    }

    fn direct_glyph_key(&self, ch: char) -> GlyphKey {
        let preferred = self.slot_for_char(ch);
        let preferred_font = self.font_for_slot(preferred).as_scaled(1.0);
        let preferred_id = preferred_font.glyph_id(ch);
        if preferred_id.0 != 0 {
            return GlyphKey::new(preferred, preferred_id.0);
        }

        let fallback = FontSlot::Latin;
        let fallback_font = self.font_for_slot(fallback).as_scaled(1.0);
        GlyphKey::new(fallback, fallback_font.glyph_id(ch).0)
    }

    fn build_atlas_from_inventory(
        &self,
        font_size: f32,
        inventory: &GlyphInventory,
    ) -> Result<FontAtlas, ab_glyph::InvalidFont> {
        build_font_atlas_from_inventory(font_size, inventory, |slot| self.font_for_slot(slot))
    }

    fn shape_run(&self, font_size: f32, run: &TextRun) -> Vec<ShapedGlyph> {
        let slot = run.font_slot;
        let face = self.face_for_slot(slot);
        let units_per_em = face.units_per_em() as f32;
        let scale = if units_per_em > 0.0 {
            font_size / units_per_em
        } else {
            1.0
        };

        let mut buffer = rustybuzz::UnicodeBuffer::new();
        buffer.push_str(&run.text);
        buffer.set_direction(match run.direction {
            TextDirection::Ltr => rustybuzz::Direction::LeftToRight,
            TextDirection::Rtl => rustybuzz::Direction::RightToLeft,
        });
        buffer.guess_segment_properties();
        let glyph_buffer = rustybuzz::shape(face, &[], buffer);

        glyph_buffer
            .glyph_infos()
            .iter()
            .zip(glyph_buffer.glyph_positions().iter())
            .map(|(info, pos)| ShapedGlyph {
                key: GlyphKey::new(slot, info.glyph_id as u16),
                x_advance: pos.x_advance as f32 * scale,
                x_offset: pos.x_offset as f32 * scale,
                y_offset: pos.y_offset as f32 * scale,
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct EmojiFontOptions {
    pub override_path: Option<PathBuf>,
    pub probe_system_fonts: bool,
}

impl Default for EmojiFontOptions {
    fn default() -> Self {
        Self {
            override_path: None,
            probe_system_fonts: true,
        }
    }
}

fn resolve_emoji_font_bytes(options: &EmojiFontOptions) -> Option<&'static [u8]> {
    let mut candidates = Vec::new();
    if let Some(path) = &options.override_path {
        candidates.push(path.clone());
    }
    if let Ok(path) = std::env::var("KAMI_EMOJI_FONT_PATH") {
        if !path.is_empty() {
            candidates.push(PathBuf::from(path));
        }
    }
    if options.probe_system_fonts {
        candidates.extend(common_emoji_font_candidates());
    }

    for path in candidates {
        if let Some(bytes) = load_emoji_font_bytes(&path) {
            return Some(bytes);
        }
    }

    None
}

fn load_emoji_font_bytes(path: &Path) -> Option<&'static [u8]> {
    let bytes = fs::read(path).ok()?;
    let font = ab_glyph::FontArc::try_from_vec(bytes.clone()).ok()?;
    let sample = font.as_scaled(64.0).glyph_id('😀');
    if sample.0 == 0 {
        return None;
    }
    if font.outline_glyph(sample.with_scale(64.0)).is_none() {
        return None;
    }
    let leaked = Box::leak(bytes.into_boxed_slice());
    Some(leaked)
}

fn common_emoji_font_candidates() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    paths.push(PathBuf::from("/System/Library/Fonts/Apple Color Emoji.ttc"));
    paths.push(PathBuf::from("/System/Library/Fonts/AppleColorEmoji.ttc"));
    paths.push(PathBuf::from(
        "/usr/share/fonts/truetype/noto/NotoColorEmoji.ttf",
    ));
    paths.push(PathBuf::from("/usr/share/fonts/noto/NotoColorEmoji.ttf"));
    paths.push(PathBuf::from("C:\\Windows\\Fonts\\seguiemj.ttf"));
    paths
}

pub struct DynamicGlyphAtlas {
    manager: FontManager,
    font_size: f32,
    capacity: usize,
    glyph_keys: BTreeSet<GlyphKey>,
    char_map: HashMap<char, GlyphKey>,
    usage_tick: u64,
    last_used: HashMap<GlyphKey, u64>,
    atlas: FontAtlas,
}

impl DynamicGlyphAtlas {
    pub fn new_default(font_size: f32, capacity: usize) -> Result<Self, ab_glyph::InvalidFont> {
        let manager = FontManager::new_default()?;
        let seed_inventory = manager.collect_glyph_inventory(font_size, " ?");
        let atlas = manager.build_atlas_from_inventory(font_size, &seed_inventory)?;
        Ok(Self {
            manager,
            font_size,
            capacity: capacity.max(32),
            glyph_keys: seed_inventory.keys,
            char_map: seed_inventory.char_map,
            usage_tick: 0,
            last_used: HashMap::new(),
            atlas,
        })
    }

    pub fn ensure_text(&mut self, text: &str) -> Result<bool, ab_glyph::InvalidFont> {
        let incoming = self.manager.collect_glyph_inventory(self.font_size, text);
        let mut changed = false;
        self.usage_tick += 1;
        for key in &incoming.keys {
            self.last_used.insert(*key, self.usage_tick);
            if self.glyph_keys.insert(*key) {
                changed = true;
            }
        }
        for (ch, key) in incoming.char_map {
            if self.char_map.insert(ch, key) != Some(key) {
                changed = true;
            }
        }

        if self.glyph_keys.len() > self.capacity {
            let pinned_space = self.manager.direct_glyph_key(' ');
            let pinned_question = self.manager.direct_glyph_key('?');
            let mut keys: Vec<GlyphKey> = self.glyph_keys.iter().copied().collect();
            keys.sort_by_key(|key| self.last_used.get(key).copied().unwrap_or_default());
            let remove_count = self.glyph_keys.len().saturating_sub(self.capacity);
            for key in keys.into_iter().take(remove_count) {
                if key != pinned_space && key != pinned_question {
                    self.glyph_keys.remove(&key);
                    self.char_map.retain(|_, mapped| *mapped != key);
                    changed = true;
                }
            }
        }

        if changed {
            let inventory = GlyphInventory {
                keys: self.glyph_keys.clone(),
                char_map: self.char_map.clone(),
            };
            self.atlas = self
                .manager
                .build_atlas_from_inventory(self.font_size, &inventory)?;
        }

        Ok(changed)
    }

    pub fn atlas(&self) -> &FontAtlas {
        &self.atlas
    }
}

/// A single glyph in the SDF atlas.
#[derive(Debug, Clone, Copy)]
pub struct Glyph {
    pub key: GlyphKey,
    pub codepoint: Option<char>,
    pub atlas_x: u16,
    pub atlas_y: u16,
    pub atlas_w: u16,
    pub atlas_h: u16,
    pub bearing_x: f32,
    pub bearing_y: f32,
    pub advance: f32,
}

/// Per-character instance data for instanced text rendering.
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct TextInstance {
    pub position: [f32; 2], // screen position
    pub uv_rect: [f32; 4],  // atlas UV (x, y, w, h)
    pub size: [f32; 2],     // quad size
    pub color: [f32; 4],    // RGBA
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorGlyphInstance {
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub uv_rect: [f32; 4],
}

#[derive(Debug, Clone)]
pub struct ColorGlyphAtlas {
    pub width: u32,
    pub height: u32,
    pub rgba_data: Vec<u8>,
    glyph_index: HashMap<String, Glyph>,
}

impl ColorGlyphAtlas {
    pub fn empty() -> Self {
        Self {
            width: 1,
            height: 1,
            rgba_data: vec![0, 0, 0, 0],
            glyph_index: HashMap::new(),
        }
    }

    pub fn glyph(&self, cluster: &str) -> Option<&Glyph> {
        self.glyph_index.get(cluster)
    }
}

pub struct DynamicColorGlyphAtlas {
    font_size: f32,
    capacity: usize,
    clusters: BTreeSet<String>,
    usage_tick: u64,
    last_used: HashMap<String, u64>,
    atlas: ColorGlyphAtlas,
}

impl DynamicColorGlyphAtlas {
    pub fn new(font_size: f32, capacity: usize) -> Self {
        let mut clusters = BTreeSet::new();
        clusters.insert("✨".to_string());
        let atlas = build_color_glyph_atlas_from_clusters(
            &clusters.iter().cloned().collect::<Vec<_>>(),
            font_size,
        );
        Self {
            font_size,
            capacity: capacity.max(8),
            clusters,
            usage_tick: 0,
            last_used: HashMap::new(),
            atlas,
        }
    }

    pub fn ensure_text(&mut self, text: &str) -> bool {
        let incoming = emoji_clusters(text);
        self.usage_tick += 1;
        let mut changed = false;

        for cluster in incoming {
            self.last_used.insert(cluster.clone(), self.usage_tick);
            if self.clusters.insert(cluster) {
                changed = true;
            }
        }

        if self.clusters.len() > self.capacity {
            let mut keys: Vec<String> = self.clusters.iter().cloned().collect();
            keys.sort_by_key(|key| self.last_used.get(key).copied().unwrap_or_default());
            let remove_count = self.clusters.len().saturating_sub(self.capacity);
            for key in keys.into_iter().take(remove_count) {
                if key != "✨" {
                    self.clusters.remove(&key);
                    self.last_used.remove(&key);
                    changed = true;
                }
            }
        }

        if changed {
            self.atlas = build_color_glyph_atlas_from_clusters(
                &self.clusters.iter().cloned().collect::<Vec<_>>(),
                self.font_size,
            );
        }

        changed
    }

    pub fn atlas(&self) -> &ColorGlyphAtlas {
        &self.atlas
    }
}

#[derive(Debug, Clone)]
struct NativeColorGlyph {
    cluster: String,
    width: u16,
    height: u16,
    advance: f32,
    baseline_from_top: f32,
    rgba: Vec<u8>,
}

#[cfg(target_os = "macos")]
#[repr(C)]
struct NativeEmojiRaster {
    rgba: *mut u8,
    len: usize,
    width: u32,
    height: u32,
    advance: f32,
    baseline_from_top: f32,
}

#[cfg(target_os = "macos")]
unsafe extern "C" {
    fn kami_render_native_emoji_rgba(
        cluster_utf8: *const std::ffi::c_char,
        font_size: f32,
        canvas_size: u32,
        out: *mut NativeEmojiRaster,
    ) -> bool;

    fn kami_free_native_emoji_rgba(ptr: *mut u8, len: usize);
}

/// SDF font atlas.
pub struct FontAtlas {
    pub width: u32,
    pub height: u32,
    pub sdf_data: Vec<u8>, // single-channel glyph alpha atlas
    pub glyphs: Vec<Glyph>,
    pub line_height: f32,
    pub ascender: f32,
    font_size: f32,
    shaped_compatible: bool,
    glyph_key_index: HashMap<GlyphKey, usize>,
    glyph_index: HashMap<char, usize>,
}

impl FontAtlas {
    /// Generate a minimal ASCII SDF atlas from procedural box glyphs.
    /// For production: use msdfgen or fontdue to generate from TTF.
    pub fn ascii_procedural(font_size: f32) -> Self {
        let glyph_w = (font_size * 0.6) as u16;
        let glyph_h = font_size as u16;
        let cols = 16u16;
        let rows = 6u16; // ASCII 32..127
        let atlas_w = (cols * glyph_w) as u32;
        let atlas_h = (rows * glyph_h) as u32;

        let mut sdf_data = vec![0u8; (atlas_w * atlas_h) as usize];
        let mut glyphs = Vec::new();
        let mut glyph_index = HashMap::new();

        for i in 0..96u16 {
            let ch = (i + 32) as u8 as char;
            let key = GlyphKey::new(FontSlot::Latin, ch as u16);
            let col = i % cols;
            let row = i / cols;
            let ax = col * glyph_w;
            let ay = row * glyph_h;

            // Simple box SDF: distance from edge
            for py in 0..glyph_h {
                for px in 0..glyph_w {
                    let dx = (px as f32 / glyph_w as f32 - 0.5).abs();
                    let dy = (py as f32 / glyph_h as f32 - 0.5).abs();
                    let d = 0.5 - dx.max(dy); // box SDF, positive inside
                    let v = ((d * 255.0 * 4.0) + 128.0).clamp(0.0, 255.0) as u8;
                    let idx = ((ay + py) as u32 * atlas_w + (ax + px) as u32) as usize;
                    if idx < sdf_data.len() {
                        sdf_data[idx] = v;
                    }
                }
            }

            glyphs.push(Glyph {
                key,
                codepoint: Some(ch),
                atlas_x: ax,
                atlas_y: ay,
                atlas_w: glyph_w,
                atlas_h: glyph_h,
                bearing_x: 0.0,
                bearing_y: font_size * 0.8,
                advance: font_size * 0.6,
            });
            glyph_index.insert(ch, glyphs.len() - 1);
        }

        let glyph_key_index = glyphs
            .iter()
            .enumerate()
            .map(|(index, glyph)| (glyph.key, index))
            .collect();

        FontAtlas {
            width: atlas_w,
            height: atlas_h,
            sdf_data,
            glyphs,
            line_height: font_size * 1.2,
            ascender: font_size * 0.8,
            font_size,
            shaped_compatible: false,
            glyph_key_index,
            glyph_index,
        }
    }

    pub fn from_ttf_bytes(ttf: &[u8], font_size: f32) -> Result<Self, ab_glyph::InvalidFont> {
        Self::from_ttf_bytes_with_charset(ttf, font_size, &basic_latin_charset())
    }

    pub fn from_ttf_bytes_with_charset(
        ttf: &[u8],
        font_size: f32,
        charset: &str,
    ) -> Result<Self, ab_glyph::InvalidFont> {
        let font = ab_glyph::FontArc::try_from_vec(ttf.to_vec())?;
        let mut keys = BTreeSet::new();
        let mut char_map = HashMap::new();
        for ch in normalized_charset(charset) {
            let key = GlyphKey::new(FontSlot::Latin, font.as_scaled(1.0).glyph_id(ch).0);
            keys.insert(key);
            char_map.insert(ch, key);
        }
        build_font_atlas_from_inventory(font_size, &GlyphInventory { keys, char_map }, |_slot| {
            &font
        })
    }

    pub fn from_default_font_stack(
        font_size: f32,
        charset: &str,
    ) -> Result<Self, ab_glyph::InvalidFont> {
        FontManager::new_default()?.build_atlas(font_size, charset)
    }

    pub fn glyph(&self, ch: char) -> Option<&Glyph> {
        self.glyph_index
            .get(&ch)
            .and_then(|index| self.glyphs.get(*index))
    }

    pub fn glyph_by_key(&self, key: GlyphKey) -> Option<&Glyph> {
        self.glyph_key_index
            .get(&key)
            .and_then(|index| self.glyphs.get(*index))
    }
}

#[derive(Debug, Clone)]
struct GlyphInventory {
    keys: BTreeSet<GlyphKey>,
    char_map: HashMap<char, GlyphKey>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RasterRun {
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub alpha: f32,
    pub color: [f32; 4],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextScript {
    Latin,
    Cjk,
    Emoji,
    Common,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextDirection {
    Ltr,
    Rtl,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextRun {
    pub script: TextScript,
    pub direction: TextDirection,
    pub font_slot: FontSlot,
    pub text: String,
}

/// Layout text into TextInstance array.
pub fn layout_text(
    atlas: &FontAtlas,
    text: &str,
    origin: Vec2,
    color: Vec4,
    scale: f32,
) -> Vec<TextInstance> {
    layout_text_shaped(atlas, text, origin, color, scale)
}

pub fn layout_text_shaped(
    atlas: &FontAtlas,
    text: &str,
    origin: Vec2,
    color: Vec4,
    scale: f32,
) -> Vec<TextInstance> {
    if atlas.shaped_compatible {
        if let Ok(manager) = FontManager::new_default() {
            return layout_text_with_manager(&manager, atlas, text, origin, color, scale);
        }
    }

    let mut instances = Vec::new();
    let mut cursor = origin;
    let aw = atlas.width as f32;
    let ah = atlas.height as f32;

    for run in segment_text_runs(text) {
        if run.text.is_empty() {
            continue;
        }
        for grapheme in run.text.graphemes(true) {
            if grapheme == "\n" {
                cursor.x = origin.x;
                cursor.y += atlas.line_height * scale;
                continue;
            }
            let cluster_origin = cursor;
            let mut cluster_advance = 0.0f32;
            let mut drew_any = false;
            for ch in grapheme.chars() {
                if let Some(g) = atlas.glyph(ch) {
                    let w = g.atlas_w as f32 * scale;
                    let h = g.atlas_h as f32 * scale;
                    let advance_cursor = if is_combining_mark(ch) {
                        cluster_origin.x
                    } else {
                        cluster_origin.x + cluster_advance
                    };
                    instances.push(TextInstance {
                        position: [
                            advance_cursor + g.bearing_x * scale,
                            cursor.y - g.bearing_y * scale,
                        ],
                        uv_rect: [
                            g.atlas_x as f32 / aw,
                            g.atlas_y as f32 / ah,
                            g.atlas_w as f32 / aw,
                            g.atlas_h as f32 / ah,
                        ],
                        size: [w, h],
                        color: color.to_array(),
                    });
                    if !is_combining_mark(ch) {
                        cluster_advance += g.advance * scale;
                    }
                    drew_any = true;
                }
            }
            if drew_any {
                cursor.x += cluster_advance.max(scale * 2.0);
            } else {
                cursor.x += scale * 8.0;
            }
        }
    }
    instances
}

fn layout_text_with_manager(
    manager: &FontManager,
    atlas: &FontAtlas,
    text: &str,
    origin: Vec2,
    color: Vec4,
    scale: f32,
) -> Vec<TextInstance> {
    let mut instances = Vec::new();
    let mut cursor = origin;
    let aw = atlas.width as f32;
    let ah = atlas.height as f32;
    let line_height = atlas.line_height * scale;
    let source_font_size = atlas.font_size.max(1.0);
    let shape_size = source_font_size * scale;

    for run in segment_text_runs(text) {
        if run.text == "\n" {
            cursor.x = origin.x;
            cursor.y += line_height;
            continue;
        }
        if run.text.is_empty() {
            continue;
        }

        let slot = run.font_slot;
        let shaped = manager.shape_run(shape_size, &run);
        for glyph in shaped {
            if let Some(atlas_glyph) = atlas.glyph_by_key(glyph.key) {
                let position_x = cursor.x + glyph.x_offset + atlas_glyph.bearing_x * scale;
                let position_y = cursor.y - glyph.y_offset - atlas_glyph.bearing_y * scale;
                instances.push(TextInstance {
                    position: [position_x, position_y],
                    uv_rect: [
                        atlas_glyph.atlas_x as f32 / aw,
                        atlas_glyph.atlas_y as f32 / ah,
                        atlas_glyph.atlas_w as f32 / aw,
                        atlas_glyph.atlas_h as f32 / ah,
                    ],
                    size: [
                        atlas_glyph.atlas_w as f32 * scale,
                        atlas_glyph.atlas_h as f32 * scale,
                    ],
                    color: color.to_array(),
                });
                cursor.x += glyph.x_advance;
            } else if let Some(fallback) = atlas.glyph_by_key(GlyphKey::new(slot, 0)) {
                instances.push(TextInstance {
                    position: [
                        cursor.x + fallback.bearing_x * scale,
                        cursor.y - fallback.bearing_y * scale,
                    ],
                    uv_rect: [
                        fallback.atlas_x as f32 / aw,
                        fallback.atlas_y as f32 / ah,
                        fallback.atlas_w as f32 / aw,
                        fallback.atlas_h as f32 / ah,
                    ],
                    size: [
                        fallback.atlas_w as f32 * scale,
                        fallback.atlas_h as f32 * scale,
                    ],
                    color: color.to_array(),
                });
                cursor.x += fallback.advance * scale;
            } else {
                cursor.x += scale * 8.0;
            }
        }
    }

    instances
}

pub fn layout_text_raster_runs(
    atlas: &FontAtlas,
    text: &str,
    origin: Vec2,
    color: Vec4,
    scale: f32,
) -> Vec<RasterRun> {
    layout_text(atlas, text, origin, color, scale)
        .into_iter()
        .map(|instance| RasterRun {
            position: instance.position,
            size: instance.size,
            alpha: instance.color[3],
            color: instance.color,
        })
        .collect()
}

pub fn build_color_glyph_atlas(text: &str, font_size: f32) -> ColorGlyphAtlas {
    let clusters = emoji_clusters(text);
    build_color_glyph_atlas_from_clusters(&clusters, font_size)
}

fn build_color_glyph_atlas_from_clusters(clusters: &[String], font_size: f32) -> ColorGlyphAtlas {
    if clusters.is_empty() {
        return ColorGlyphAtlas::empty();
    }

    let native = native_color_glyphs(&clusters, font_size);

    let glyph_w = (font_size * 1.4).ceil().max(24.0) as u16;
    let glyph_h = (font_size * 1.4).ceil().max(24.0) as u16;
    let cols = 8u16;
    let rows = (clusters.len() as u16).div_ceil(cols).max(1);
    let width = (cols * glyph_w) as u32;
    let height = (rows * glyph_h) as u32;
    let mut rgba = vec![0u8; (width * height * 4) as usize];
    let mut glyph_index = HashMap::new();

    for (i, cluster) in clusters.iter().enumerate() {
        let col = i as u16 % cols;
        let row = i as u16 / cols;
        let cell_x = col * glyph_w;
        let cell_y = row * glyph_h;
        if let Some(native) = native.iter().find(|glyph| glyph.cluster == *cluster) {
            let placement =
                blit_color_glyph_rgba(&mut rgba, width, cell_x, cell_y, glyph_w, glyph_h, native);
            let bearing_y = native.baseline_from_top - placement.min_y as f32;
            glyph_index.insert(
                cluster.clone(),
                Glyph {
                    key: GlyphKey::new(FontSlot::Emoji, i as u16 + 1),
                    codepoint: cluster.chars().next(),
                    atlas_x: cell_x + placement.offset_x,
                    atlas_y: cell_y + placement.offset_y,
                    atlas_w: placement.width,
                    atlas_h: placement.height,
                    bearing_x: 0.0,
                    bearing_y,
                    advance: native.advance.max(placement.width as f32),
                },
            );
        } else {
            procedural_color_glyph_rgba(
                &mut rgba, width, cell_x, cell_y, glyph_w, glyph_h, cluster,
            );
            glyph_index.insert(
                cluster.clone(),
                Glyph {
                    key: GlyphKey::new(FontSlot::Emoji, i as u16 + 1),
                    codepoint: cluster.chars().next(),
                    atlas_x: cell_x,
                    atlas_y: cell_y,
                    atlas_w: glyph_w,
                    atlas_h: glyph_h,
                    bearing_x: 0.0,
                    bearing_y: glyph_h as f32 * 0.85,
                    advance: glyph_w as f32 * 0.92,
                },
            );
        }
    }

    ColorGlyphAtlas {
        width,
        height,
        rgba_data: rgba,
        glyph_index,
    }
}

pub fn layout_color_glyphs(
    atlas: &ColorGlyphAtlas,
    text: &str,
    origin: Vec2,
    line_height: f32,
    scale: f32,
) -> Vec<ColorGlyphInstance> {
    let mut instances = Vec::new();
    let mut cursor = origin;
    let aw = atlas.width as f32;
    let ah = atlas.height as f32;

    for grapheme in text.graphemes(true) {
        if grapheme == "\n" {
            cursor.x = origin.x;
            cursor.y += line_height * scale;
            continue;
        }
        if prefers_emoji_cluster(grapheme) {
            if let Some(glyph) = atlas.glyph(grapheme) {
                instances.push(ColorGlyphInstance {
                    position: [
                        cursor.x + glyph.bearing_x * scale,
                        cursor.y - glyph.bearing_y * scale,
                    ],
                    size: [glyph.atlas_w as f32 * scale, glyph.atlas_h as f32 * scale],
                    uv_rect: [
                        glyph.atlas_x as f32 / aw,
                        glyph.atlas_y as f32 / ah,
                        glyph.atlas_w as f32 / aw,
                        glyph.atlas_h as f32 / ah,
                    ],
                });
                cursor.x += glyph.advance * scale;
                continue;
            }
        }
        cursor.x += line_height * 0.35 * scale;
    }

    instances
}

fn basic_latin_charset() -> String {
    (32u8..=126u8).map(char::from).collect()
}

pub fn segment_text_runs(text: &str) -> Vec<TextRun> {
    let mut runs = Vec::new();
    for paragraph in text.split_inclusive('\n') {
        let has_newline = paragraph.ends_with('\n');
        let body = paragraph.strip_suffix('\n').unwrap_or(paragraph);
        for run in segment_paragraph_runs(body) {
            runs.push(run);
        }
        if has_newline {
            runs.push(TextRun {
                script: TextScript::Common,
                direction: TextDirection::Ltr,
                font_slot: FontSlot::Latin,
                text: "\n".to_string(),
            });
        }
    }
    runs
}

fn segment_paragraph_runs(text: &str) -> Vec<TextRun> {
    if text.is_empty() {
        return Vec::new();
    }

    let visual_text = reorder_visual_text(text);
    let mut runs = Vec::new();
    let mut current = String::new();
    let mut current_script = None;
    let mut current_direction = None;
    let mut current_slot = None;

    for grapheme in visual_text.graphemes(true) {
        let script = script_for_cluster(grapheme);
        let direction = direction_for_cluster(grapheme);
        let font_slot = font_slot_for_cluster(grapheme);

        match (current_script, current_direction, current_slot) {
            (Some(active_script), Some(active_direction), Some(active_slot))
                if active_script == script
                    && active_direction == direction
                    && active_slot == font_slot =>
            {
                current.push_str(grapheme);
            }
            (Some(active_script), Some(active_direction), Some(active_slot)) => {
                runs.push(TextRun {
                    script: active_script,
                    direction: active_direction,
                    font_slot: active_slot,
                    text: std::mem::take(&mut current),
                });
                current.push_str(grapheme);
                current_script = Some(script);
                current_direction = Some(direction);
                current_slot = Some(font_slot);
            }
            _ => {
                current.push_str(grapheme);
                current_script = Some(script);
                current_direction = Some(direction);
                current_slot = Some(font_slot);
            }
        }
    }

    if !current.is_empty() {
        runs.push(TextRun {
            script: current_script.unwrap_or(TextScript::Common),
            direction: current_direction.unwrap_or(TextDirection::Ltr),
            font_slot: current_slot.unwrap_or(FontSlot::Latin),
            text: current,
        });
    }

    runs
}

fn reorder_visual_text(text: &str) -> String {
    let bidi = unicode_bidi::BidiInfo::new(text, None);
    let Some(paragraph) = bidi.paragraphs.first() else {
        return text.to_string();
    };
    bidi.reorder_line(paragraph, paragraph.range.clone())
        .to_string()
}

fn prefers_cjk_font(ch: char) -> bool {
    matches!(
        ch as u32,
        0x3000..=0x303F
            | 0x3040..=0x309F
            | 0x30A0..=0x30FF
            | 0x31F0..=0x31FF
            | 0x3400..=0x4DBF
            | 0x4E00..=0x9FFF
            | 0xF900..=0xFAFF
            | 0xFF00..=0xFFEF
    )
}

fn prefers_emoji_font(ch: char) -> bool {
    matches!(
        ch as u32,
        0x1F1E6..=0x1F1FF
            | 0x1F300..=0x1FAFF
            | 0x2600..=0x27BF
            | 0xFE0F
            | 0x200D
    )
}

fn prefers_emoji_cluster(cluster: &str) -> bool {
    cluster.chars().any(prefers_emoji_font)
}

fn emoji_clusters(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    for grapheme in text.graphemes(true) {
        if prefers_emoji_cluster(grapheme) && seen.insert(grapheme.to_string()) {
            out.push(grapheme.to_string());
        }
    }
    out
}

fn procedural_color_glyph_rgba(
    rgba: &mut [u8],
    atlas_width: u32,
    cell_x: u16,
    cell_y: u16,
    glyph_w: u16,
    glyph_h: u16,
    cluster: &str,
) {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    cluster.hash(&mut hasher);
    let seed = hasher.finish();
    let palette = [
        [
            ((seed >> 0) & 0xff) as u8,
            ((seed >> 8) & 0xff) as u8,
            ((seed >> 16) & 0xff) as u8,
        ],
        [
            ((seed >> 24) & 0xff) as u8,
            ((seed >> 32) & 0xff) as u8,
            ((seed >> 40) & 0xff) as u8,
        ],
        [
            ((seed >> 12) & 0xff) as u8,
            ((seed >> 20) & 0xff) as u8,
            ((seed >> 28) & 0xff) as u8,
        ],
    ];
    let radius = glyph_w.min(glyph_h) as f32 * 0.42;
    let cx = glyph_w as f32 * 0.5;
    let cy = glyph_h as f32 * 0.5;

    for y in 0..glyph_h {
        for x in 0..glyph_w {
            let px = cell_x as u32 + x as u32;
            let py = cell_y as u32 + y as u32;
            let idx = ((py * atlas_width + px) * 4) as usize;
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist > radius {
                continue;
            }
            let t = (x as f32 / glyph_w as f32).clamp(0.0, 1.0);
            let band = (((y as f32 / glyph_h as f32) * 3.0).floor() as usize).min(2);
            let blend = [
                (palette[band][0] as f32 * (1.0 - t) + 255.0 * t * 0.25) as u8,
                (palette[band][1] as f32 * (1.0 - t) + 255.0 * t * 0.2) as u8,
                (palette[band][2] as f32 * (1.0 - t) + 255.0 * t * 0.15) as u8,
            ];
            rgba[idx] = blend[0];
            rgba[idx + 1] = blend[1];
            rgba[idx + 2] = blend[2];
            rgba[idx + 3] = 255;
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct GlyphPlacement {
    offset_x: u16,
    offset_y: u16,
    min_y: u16,
    width: u16,
    height: u16,
}

fn blit_color_glyph_rgba(
    rgba: &mut [u8],
    atlas_width: u32,
    cell_x: u16,
    cell_y: u16,
    glyph_w: u16,
    glyph_h: u16,
    native: &NativeColorGlyph,
) -> GlyphPlacement {
    let Some((min_x, min_y, max_x, max_y)) = color_bounds(native) else {
        return GlyphPlacement {
            offset_x: 0,
            offset_y: 0,
            min_y: 0,
            width: glyph_w,
            height: glyph_h,
        };
    };
    let src_w = (max_x - min_x + 1) as usize;
    let src_h = (max_y - min_y + 1) as usize;
    let copy_w = src_w.min(glyph_w as usize);
    let copy_h = src_h.min(glyph_h as usize);
    let offset_x = ((glyph_w as i32 - copy_w as i32).max(0) / 2) as usize;
    let offset_y = ((glyph_h as i32 - copy_h as i32).max(0) / 2) as usize;

    for y in 0..copy_h {
        for x in 0..copy_w {
            let src_idx =
                (((min_y as usize + y) * native.width as usize) + (min_x as usize + x)) * 4;
            let dst_x = cell_x as usize + offset_x + x;
            let dst_y = cell_y as usize + offset_y + y;
            let dst_idx = ((dst_y as u32 * atlas_width + dst_x as u32) * 4) as usize;
            rgba[dst_idx..dst_idx + 4].copy_from_slice(&native.rgba[src_idx..src_idx + 4]);
        }
    }

    GlyphPlacement {
        offset_x: offset_x as u16,
        offset_y: offset_y as u16,
        min_y: min_y as u16,
        width: copy_w as u16,
        height: copy_h as u16,
    }
}

fn color_bounds(native: &NativeColorGlyph) -> Option<(u32, u32, u32, u32)> {
    let mut min_x = native.width as u32;
    let mut min_y = native.height as u32;
    let mut max_x = 0u32;
    let mut max_y = 0u32;
    let mut found = false;
    for y in 0..native.height as u32 {
        for x in 0..native.width as u32 {
            let idx = ((y * native.width as u32 + x) * 4 + 3) as usize;
            if native.rgba.get(idx).copied().unwrap_or(0) > 0 {
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
                found = true;
            }
        }
    }
    found.then_some((min_x, min_y, max_x, max_y))
}

fn native_color_glyphs(clusters: &[String], font_size: f32) -> Vec<NativeColorGlyph> {
    #[cfg(target_os = "macos")]
    {
        clusters
            .iter()
            .filter_map(|cluster| render_native_macos_emoji(cluster, font_size))
            .collect()
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (clusters, font_size);
        Vec::new()
    }
}

#[cfg(target_os = "macos")]
fn render_native_macos_emoji(cluster: &str, font_size: f32) -> Option<NativeColorGlyph> {
    let canvas_size = (font_size * 2.2).ceil().max(64.0) as u32;
    let cluster_c = std::ffi::CString::new(cluster).ok()?;
    let mut raster = NativeEmojiRaster {
        rgba: std::ptr::null_mut(),
        len: 0,
        width: 0,
        height: 0,
        advance: 0.0,
        baseline_from_top: 0.0,
    };

    let ok = unsafe {
        kami_render_native_emoji_rgba(cluster_c.as_ptr(), font_size, canvas_size, &mut raster)
    };
    if !ok || raster.rgba.is_null() || raster.len == 0 || raster.width == 0 || raster.height == 0 {
        return None;
    }

    let rgba = unsafe { std::slice::from_raw_parts(raster.rgba, raster.len).to_vec() };
    unsafe {
        kami_free_native_emoji_rgba(raster.rgba, raster.len);
    }

    Some(NativeColorGlyph {
        cluster: cluster.to_string(),
        width: raster.width as u16,
        height: raster.height as u16,
        advance: raster.advance,
        baseline_from_top: raster.baseline_from_top,
        rgba,
    })
}

fn script_for_char(ch: char) -> TextScript {
    if prefers_emoji_font(ch) {
        TextScript::Emoji
    } else if prefers_cjk_font(ch) {
        TextScript::Cjk
    } else if ch.is_ascii_alphanumeric() || ch.is_ascii_punctuation() || ch.is_whitespace() {
        TextScript::Latin
    } else {
        TextScript::Common
    }
}

fn script_for_cluster(grapheme: &str) -> TextScript {
    grapheme
        .chars()
        .find(|ch| !is_combining_mark(*ch))
        .map(script_for_char)
        .unwrap_or(TextScript::Common)
}

fn direction_for_cluster(grapheme: &str) -> TextDirection {
    if grapheme.chars().any(is_rtl_char) {
        TextDirection::Rtl
    } else {
        TextDirection::Ltr
    }
}

fn font_slot_for_cluster(grapheme: &str) -> FontSlot {
    if grapheme.chars().any(prefers_emoji_font) {
        FontSlot::Emoji
    } else if grapheme.chars().any(prefers_cjk_font) {
        FontSlot::Cjk
    } else {
        FontSlot::Latin
    }
}

fn is_combining_mark(ch: char) -> bool {
    matches!(
        ch as u32,
        0x0300..=0x036F
            | 0x1AB0..=0x1AFF
            | 0x1DC0..=0x1DFF
            | 0x20D0..=0x20FF
            | 0xFE20..=0xFE2F
    )
}

fn normalized_charset(input: &str) -> Vec<char> {
    let mut set = BTreeSet::new();
    set.insert(' ');
    set.insert('?');
    for ch in input.chars() {
        if !ch.is_control() || ch == '\n' || ch == '\t' {
            set.insert(ch);
        }
    }
    set.into_iter().collect()
}

fn is_rtl_char(ch: char) -> bool {
    matches!(
        ch as u32,
        0x0590..=0x08FF
            | 0xFB1D..=0xFDFF
            | 0xFE70..=0xFEFF
            | 0x10800..=0x10FFF
    )
}

fn build_font_atlas_from_inventory<'a, F>(
    font_size: f32,
    inventory: &GlyphInventory,
    select_font: F,
) -> Result<FontAtlas, ab_glyph::InvalidFont>
where
    F: Fn(FontSlot) -> &'a ab_glyph::FontArc,
{
    let mut ordered_keys: Vec<GlyphKey> = inventory.keys.iter().copied().collect();
    if ordered_keys.is_empty() {
        ordered_keys.push(GlyphKey::new(FontSlot::Latin, 0));
    }
    let glyph_h = (font_size * 1.55).ceil() as u16;
    let glyph_w = (font_size * 1.20).ceil() as u16;
    let cols = 16u16;
    let rows = (ordered_keys.len() as u16).div_ceil(cols).max(1);
    let atlas_w = (cols * glyph_w) as u32;
    let atlas_h = (rows * glyph_h) as u32;
    let mut atlas = vec![0u8; (atlas_w * atlas_h) as usize];
    let mut glyphs = Vec::new();
    let mut glyph_key_index = HashMap::new();
    let mut glyph_index = HashMap::new();

    let latin_metrics = select_font(FontSlot::Latin).as_scaled(font_size);
    let cjk_metrics = select_font(FontSlot::Cjk).as_scaled(font_size);
    let ascent = latin_metrics.ascent().max(cjk_metrics.ascent());
    let descent = latin_metrics
        .descent()
        .abs()
        .max(cjk_metrics.descent().abs());
    let line_gap = latin_metrics.line_gap().max(cjk_metrics.line_gap());

    for (i, key) in ordered_keys.iter().copied().enumerate() {
        let active_font = select_font(key.slot());
        let scaled = active_font.as_scaled(font_size);
        let col = i as u16 % cols;
        let row = i as u16 / cols;
        let cell_x = col * glyph_w;
        let cell_y = row * glyph_h;

        let mut glyph = GlyphId(key.glyph_id).with_scale(font_size);
        glyph.position = ab_glyph::point(cell_x as f32, cell_y as f32 + ascent);

        let mut bearing_x = 0.0;
        let mut bearing_y = ascent;
        let mut atlas_glyph_w = (font_size * 0.2).max(1.0) as u16;
        let mut atlas_glyph_h = (font_size * 0.8).max(1.0) as u16;

        if let Some(outlined) = active_font.outline_glyph(glyph) {
            let bounds = outlined.px_bounds();
            bearing_x = bounds.min.x - cell_x as f32;
            bearing_y = ascent - (bounds.max.y - cell_y as f32);
            atlas_glyph_w = bounds.width().ceil().max(1.0) as u16;
            atlas_glyph_h = bounds.height().ceil().max(1.0) as u16;
            outlined.draw(|x, y, coverage| {
                let px = cell_x as u32 + x;
                let py = cell_y as u32 + y;
                let idx = (py * atlas_w + px) as usize;
                if idx < atlas.len() {
                    atlas[idx] = atlas[idx].max((coverage * 255.0).round().clamp(0.0, 255.0) as u8);
                }
            });
        }

        let codepoint = inventory
            .char_map
            .iter()
            .find_map(|(ch, mapped)| if *mapped == key { Some(*ch) } else { None });
        glyphs.push(Glyph {
            key,
            codepoint,
            atlas_x: cell_x,
            atlas_y: cell_y,
            atlas_w: atlas_glyph_w,
            atlas_h: atlas_glyph_h,
            bearing_x,
            bearing_y,
            advance: scaled.h_advance(GlyphId(key.glyph_id)),
        });
        glyph_key_index.insert(key, glyphs.len() - 1);
        if let Some(ch) = codepoint {
            glyph_index.entry(ch).or_insert(glyphs.len() - 1);
        }
    }

    Ok(FontAtlas {
        width: atlas_w,
        height: atlas_h,
        sdf_data: atlas,
        glyphs,
        line_height: ascent + descent + line_gap,
        ascender: ascent,
        font_size,
        shaped_compatible: true,
        glyph_key_index,
        glyph_index,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ascii_atlas() {
        let atlas = FontAtlas::ascii_procedural(32.0);
        assert_eq!(atlas.glyphs.len(), 96);
        assert!(atlas.glyph('A').is_some());
        assert!(atlas.glyph('z').is_some());
        assert!(atlas.sdf_data.len() > 0);
    }

    #[test]
    fn test_layout() {
        let atlas = FontAtlas::ascii_procedural(16.0);
        let instances = layout_text(&atlas, "Hello", Vec2::ZERO, Vec4::ONE, 1.0);
        assert_eq!(instances.len(), 5);
        assert!(instances[1].position[0] > instances[0].position[0]);
    }

    #[test]
    fn ttf_raster_layout_produces_runs() {
        let atlas = FontAtlas::from_default_font_stack(18.0, "Disk").expect("ttf atlas");
        let runs = layout_text_raster_runs(&atlas, "Disk", Vec2::ZERO, Vec4::ONE, 1.0);
        assert!(!runs.is_empty());
        assert!(runs.iter().any(|run| run.size[0] > 1.0));
    }

    #[test]
    fn ttf_atlas_supports_unicode_charset() {
        let atlas =
            FontAtlas::from_default_font_stack(18.0, "Disk Cleaner 日本語").expect("unicode atlas");
        assert!(atlas.glyph('日').is_some());
        assert!(atlas.glyph('D').is_some());
        let instances = layout_text(&atlas, "日本語", Vec2::ZERO, Vec4::ONE, 1.0);
        assert!(!instances.is_empty());
    }

    #[test]
    fn dynamic_glyph_atlas_expands_for_unicode_text() {
        let mut atlas = DynamicGlyphAtlas::new_default(18.0, 256).expect("dynamic atlas");
        atlas.ensure_text("Poppins 日本語").expect("ensure text");
        assert!(atlas.atlas().glyph('P').is_some());
        assert!(atlas.atlas().glyph('日').is_some());
    }

    #[test]
    fn dynamic_color_glyph_atlas_expands_for_emoji_clusters() {
        let mut atlas = DynamicColorGlyphAtlas::new(18.0, 16);
        assert!(atlas.ensure_text("Disk Cleaner ✨👨‍👩‍👧‍👦"));
        assert!(atlas.atlas().glyph("✨").is_some());
        assert!(atlas.atlas().glyph("👨‍👩‍👧‍👦").is_some());
    }

    #[test]
    fn segmenter_splits_mixed_script_runs() {
        let runs = segment_text_runs("Disk 日本語 Mix");
        assert!(runs.len() >= 3);
        assert_eq!(runs[0].script, TextScript::Latin);
        assert!(runs.iter().any(|run| run.script == TextScript::Cjk));
        assert!(runs.iter().any(|run| run.font_slot == FontSlot::Cjk));
    }

    #[test]
    fn shaped_layout_keeps_combining_mark_in_same_cluster() {
        let atlas = FontAtlas::from_default_font_stack(18.0, "e\u{0301}").expect("atlas");
        let instances = layout_text_shaped(&atlas, "e\u{0301}", Vec2::ZERO, Vec4::ONE, 1.0);
        assert!(!instances.is_empty());
        assert!(instances.len() >= 1);
        if instances.len() >= 2 {
            assert!((instances[1].position[0] - instances[0].position[0]).abs() < 8.0);
        }
    }

    #[test]
    fn shaped_inventory_resolves_glyph_ids_into_atlas_entries() {
        let manager = FontManager::new_default().expect("font manager");
        let atlas = FontAtlas::from_default_font_stack(18.0, "office 日本語").expect("atlas");
        for run in segment_text_runs("office 日本語") {
            if run.text == "\n" || run.text.is_empty() {
                continue;
            }
            let shaped = manager.shape_run(18.0, &run);
            assert!(!shaped.is_empty());
            for glyph in shaped {
                assert!(
                    atlas.glyph_by_key(glyph.key).is_some(),
                    "missing shaped glyph {:?} from atlas",
                    glyph.key
                );
            }
        }
    }

    #[test]
    fn bidi_segmenter_marks_rtl_runs() {
        let runs = segment_text_runs("abc שלום 123");
        assert!(runs.iter().any(|run| run.direction == TextDirection::Rtl));
        assert!(runs.iter().any(|run| run.direction == TextDirection::Ltr));
    }

    #[test]
    fn emoji_zwj_cluster_stays_in_single_run() {
        let runs = segment_text_runs("family 👨‍👩‍👧‍👦 ok");
        assert!(runs.iter().any(|run| run.script == TextScript::Emoji));
        assert!(runs.iter().any(|run| run.text.contains("👨‍👩‍👧‍👦")));
    }

    #[test]
    fn emoji_override_path_rejects_non_emoji_font() {
        let path = PathBuf::from(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/fonts/Poppins-Regular.ttf"
        ));
        let manager = FontManager::new_with_options(EmojiFontOptions {
            override_path: Some(path),
            probe_system_fonts: false,
        })
        .expect("font manager");
        assert!(!manager.has_emoji_font());
    }
}
