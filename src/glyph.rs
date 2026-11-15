//! Stroke shapes for every component and the block layout that assembles
//! them into one square glyph per syllable.
//!
//! All geometry lives in a 100 × 100 em box. Components are drawn as
//! centre-line strokes (round caps/joins, constant width, like a 圆体 /
//! rounded-Hei typeface) in a unit box, then mapped into the rectangle the
//! layout assigns them. Because the stroke width is applied after mapping,
//! squashing a component into a flat slot keeps every stroke the same weight,
//! just as 土 keeps its weight whether it is the full character or the
//! bottom of 坐.

use crate::phonology::{Coda, Consonant, Manner, Place, Syllable, Tone, Vowel};
use std::fmt::Write;

/// Stroke width for components.
pub const SW: f32 = 5.6;
/// Stroke width for tone marks.
pub const TONE_SW: f32 = 4.6;

#[derive(Clone, Copy, Debug)]
pub enum Seg {
    M(f32, f32),
    L(f32, f32),
    Q(f32, f32, f32, f32),
}
use Seg::{L, M, Q};

type Strokes = &'static [&'static [Seg]];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Rect { x, y, w, h }
    }

    /// Shrink to stay within an aspect-ratio range (w / h), centred.
    pub fn fit(self, min_a: f32, max_a: f32) -> Rect {
        let a = self.w / self.h;
        if a > max_a {
            let w = self.h * max_a;
            Rect { x: self.x + (self.w - w) / 2.0, w, ..self }
        } else if a < min_a {
            let h = self.w / min_a;
            Rect { y: self.y + (self.h - h) / 2.0, h, ..self }
        } else {
            self
        }
    }

    pub fn inset(self, d: f32) -> Rect {
        let dx = d.min(self.w / 2.0 - 0.5).max(0.0);
        let dy = d.min(self.h / 2.0 - 0.5).max(0.0);
        Rect { x: self.x + dx, y: self.y + dy, w: self.w - 2.0 * dx, h: self.h - 2.0 * dy }
    }

    /// A sub-rectangle given in unit coordinates of this one.
    pub fn sub(self, ux: f32, uy: f32, uw: f32, uh: f32) -> Rect {
        Rect { x: self.x + ux * self.w, y: self.y + uy * self.h, w: uw * self.w, h: uh * self.h }
    }

    fn pt(self, ux: f32, uy: f32) -> (f32, f32) {
        (self.x + ux * self.w, self.y + uy * self.h)
    }
}

fn emit(strokes: Strokes, g: Rect, out: &mut String) {
    for stroke in strokes {
        for seg in stroke.iter() {
            let _ = match *seg {
                M(x, y) => {
                    let (a, b) = g.pt(x, y);
                    write!(out, "M{a:.1} {b:.1}")
                }
                L(x, y) => {
                    let (a, b) = g.pt(x, y);
                    write!(out, "L{a:.1} {b:.1}")
                }
                Q(cx, cy, x, y) => {
                    let (c1, c2) = g.pt(cx, cy);
                    let (a, b) = g.pt(x, y);
                    write!(out, "Q{c1:.1} {c2:.1} {a:.1} {b:.1}")
                }
            };
        }
    }
}

// ---------------------------------------------------------------------------
// Consonant frames (place of articulation)
//
// The mouth is pictured facing left: left = lips, top = roof of the mouth,
// right = throat. Each frame is one or two strokes drawn along the surface
// where the closure is made.
// ---------------------------------------------------------------------------

/// 匚 — labial: closed at the front, where the lips meet.
const FANG: Strokes = &[
    &[M(0.0, 0.0), L(1.0, 0.0)],
    &[M(0.0, 0.0), L(0.0, 1.0), L(1.0, 1.0)],
];

/// 厂 — alveolar: closed at the top-front, tongue tip on the gum ridge.
const HAN: Strokes = &[
    &[M(0.04, 0.0), L(1.0, 0.0)],
    &[M(0.1, 0.0), L(0.1, 0.56), Q(0.1, 0.88, 0.0, 1.0)],
];

/// 凵 — dental: closed below, tongue tip down behind the lower teeth.
const KAN: Strokes = &[
    &[M(0.0, 0.0), L(0.0, 1.0), L(1.0, 1.0)],
    &[M(1.0, 0.0), L(1.0, 1.0)],
];

/// 乚 — retroflex: one stroke that curls up, like the tongue tip.
const YIN: Strokes = &[&[
    M(0.04, 0.0),
    L(0.04, 0.74),
    Q(0.04, 1.0, 0.32, 1.0),
    L(0.88, 1.0),
    L(0.98, 0.8),
]];

/// 冂 — palatal: a roof over everything, tongue body on the hard palate.
const JIONG: Strokes = &[
    &[M(0.0, 0.0), L(0.0, 1.0)],
    &[M(0.0, 0.0), L(1.0, 0.0), L(1.0, 1.0), L(0.84, 0.9)],
];

/// 𠃌 — velar: one 横折钩 stroke, closed at the top-back.
const GOU: Strokes = &[&[M(0.0, 0.0), L(1.0, 0.0), L(1.0, 1.0), L(0.8, 0.88)]];

fn frame(place: Place) -> (Strokes, Rect) {
    // (strokes, inner area for the manner mark in unit coordinates)
    match place {
        Place::Labial => (FANG, Rect::new(0.24, 0.2, 0.64, 0.6)),
        Place::Alveolar => (HAN, Rect::new(0.32, 0.22, 0.62, 0.66)),
        Place::Dental => (KAN, Rect::new(0.2, 0.14, 0.6, 0.64)),
        Place::Retroflex => (YIN, Rect::new(0.3, 0.1, 0.6, 0.66)),
        Place::Palatal => (JIONG, Rect::new(0.2, 0.24, 0.6, 0.62)),
        Place::Velar => (GOU, Rect::new(0.06, 0.24, 0.68, 0.6)),
    }
}

// ---------------------------------------------------------------------------
// Manner marks: always exactly one basic stroke
// ---------------------------------------------------------------------------

/// 一 aspirated
const MARK_ASP: Strokes = &[&[M(0.0, 0.5), L(1.0, 0.5)]];
/// 丶 nasal
const MARK_NAS: Strokes = &[&[M(0.28, 0.18), L(0.72, 0.82)]];
/// 丿 fricative
const MARK_FRIC: Strokes = &[&[M(0.9, 0.04), Q(0.7, 0.62, 0.06, 0.96)]];
/// 丨 liquid
const MARK_LIQ: Strokes = &[&[M(0.5, 0.0), L(0.5, 1.0)]];

fn mark(manner: Manner) -> Option<(Strokes, f32, f32)> {
    match manner {
        Manner::Plain => None,
        Manner::Aspirated => Some((MARK_ASP, 0.0, 99.0)),
        Manner::Nasal => Some((MARK_NAS, 0.62, 0.9)),
        Manner::Fricative => Some((MARK_FRIC, 0.5, 0.95)),
        Manner::Liquid => Some((MARK_LIQ, 0.0, 99.0)),
    }
}

// ---------------------------------------------------------------------------
// Vowels: one or two strokes; a dot (丶) marks the front vowels ê and ü
// ---------------------------------------------------------------------------

/// 八 a
const BA: Strokes = &[
    &[M(0.4, 0.04), Q(0.38, 0.62, 0.02, 0.96)],
    &[M(0.6, 0.1), Q(0.66, 0.62, 0.98, 0.94)],
];

/// 又 o
const YOU: Strokes = &[
    &[M(0.08, 0.04), L(0.86, 0.04), Q(0.66, 0.62, 0.04, 1.0)],
    &[M(0.22, 0.3), Q(0.5, 0.86, 0.98, 0.98)],
];

/// 了 e
const LE: Strokes = &[
    &[M(0.1, 0.04), L(0.86, 0.04), L(0.52, 0.34)],
    &[M(0.52, 0.34), L(0.52, 1.0), L(0.32, 0.88)],
];

/// 了 + 丶 ê
const LE_DOT: Strokes = &[
    &[M(0.1, 0.04), L(0.86, 0.04), L(0.52, 0.34)],
    &[M(0.52, 0.34), L(0.52, 1.0), L(0.32, 0.88)],
    &[M(0.76, 0.46), L(0.92, 0.66)],
];

/// 乙 i
const YI: Strokes = &[&[
    M(0.08, 0.04),
    L(0.86, 0.04),
    L(0.12, 0.72),
    Q(0.0, 1.0, 0.3, 1.0),
    L(0.86, 1.0),
    L(0.96, 0.8),
]];

/// 乂 u
const WU: Strokes = &[
    &[M(0.9, 0.04), Q(0.62, 0.6, 0.04, 0.96)],
    &[M(0.1, 0.06), Q(0.42, 0.6, 0.96, 0.96)],
];

/// 义 ü: 乂 with the fronting dot
const YU: Strokes = &[
    &[M(0.42, 0.0), L(0.58, 0.2)],
    &[M(0.9, 0.3), Q(0.62, 0.7, 0.04, 0.98)],
    &[M(0.1, 0.32), Q(0.42, 0.7, 0.96, 0.98)],
];

fn vowel_strokes(v: Vowel) -> Strokes {
    match v {
        Vowel::A => BA,
        Vowel::O => YOU,
        Vowel::E => LE,
        Vowel::Ehat => LE_DOT,
        Vowel::I => YI,
        Vowel::U => WU,
        Vowel::Yu => YU,
    }
}

// ---------------------------------------------------------------------------
// Tone marks (absolute coordinates, top centre)
// ---------------------------------------------------------------------------

fn tone_path(t: Tone) -> Option<String> {
    let d = match t {
        // 横: level
        Tone::First => "M40 9.5L60 9.5",
        // 提: rising from bottom-left to top-right, like pinyin ´
        Tone::Second => "M44.5 15L55.5 3.5",
        // 点 + 提: down then up, like pinyin ˇ
        Tone::Third => "M41 4L50 14.5L59 4",
        // 点: falling from top-left to bottom-right, like pinyin `
        Tone::Fourth => "M44.5 3.5L55.5 15",
        Tone::Neutral => return None,
    };
    Some(d.to_string())
}

// ---------------------------------------------------------------------------
// Layout
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Role {
    Tone,
    Initial,
    Medial,
    Nucleus,
    Coda,
    Rhotic,
}

impl Role {
    pub fn class(self) -> &'static str {
        match self {
            Role::Tone => "r-tone",
            Role::Initial => "r-initial",
            Role::Medial => "r-medial",
            Role::Nucleus => "r-nucleus",
            Role::Coda => "r-coda",
            Role::Rhotic => "r-rhotic",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Role::Tone => "Tone",
            Role::Initial => "Initial",
            Role::Medial => "Medial",
            Role::Nucleus => "Vowel",
            Role::Coda => "Coda",
            Role::Rhotic => "儿化",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Layer {
    pub role: Role,
    pub d: String,
    pub width: f32,
}

/// Aspect-ratio windows (width / height) per slot.
mod aspect {
    pub const INITIAL_SIDE: (f32, f32) = (0.4, 0.86);
    pub const INITIAL_ALONE: (f32, f32) = (0.66, 0.92);
    pub const NUCLEUS_STACKED: (f32, f32) = (0.7, 1.7);
    pub const NUCLEUS_SIDE: (f32, f32) = (0.55, 1.0);
    pub const NUCLEUS_ALONE: (f32, f32) = (0.72, 1.06);
    pub const GLIDE: (f32, f32) = (1.6, 3.4);
    pub const FRAME_CODA: (f32, f32) = (1.5, 3.0);
    pub const SPLIT: (f32, f32) = (0.7, 1.6);
}

/// Draw a consonant (frame + manner mark) into `r`, honouring `asp`.
fn consonant_path(c: Consonant, r: Rect, asp: (f32, f32)) -> String {
    let mut out = String::new();
    let (strokes, inner) = frame(c.place);
    let g = r.fit(asp.0, asp.1).inset(SW / 2.0);
    emit(strokes, g, &mut out);
    if let Some((m, lo, hi)) = mark(c.manner) {
        let ir = g.sub(inner.x, inner.y, inner.w, inner.h).fit(lo, hi);
        emit(m, ir, &mut out);
    }
    out
}

fn vowel_path(v: Vowel, r: Rect, asp: (f32, f32)) -> String {
    let mut out = String::new();
    emit(vowel_strokes(v), r.fit(asp.0, asp.1).inset(SW / 2.0), &mut out);
    out
}

/// 儿化 / er: the bare retroflex frame 乚 in the coda slot.
fn rhotic_path(r: Rect, asp: (f32, f32)) -> String {
    consonant_path(Consonant::new(Place::Retroflex, Manner::Plain), r, asp)
}

/// A nasal coda is the bare frame of its place: only n, ng (and m in
/// interjections) can close a syllable, so the nasal mark is not needed.
fn coda_frame(c: Consonant) -> Consonant {
    Consonant::new(c.place, Manner::Plain)
}

/// How flat a glide may be drawn: crossed strokes (乂, 义) stop reading as a
/// cross when squashed too far, 乙 survives more.
fn glide_aspect(v: Vowel) -> (f32, f32) {
    match v {
        Vowel::I => aspect::GLIDE,
        _ => (aspect::GLIDE.0, 2.3),
    }
}

fn layer(role: Role, d: String) -> Layer {
    Layer { role, d, width: SW }
}

const BODY_TOP: f32 = 21.0;
const BODY_H: f32 = 76.0;

/// Lay out one syllable as a list of stroke layers.
pub fn syllable_layers(s: &Syllable) -> Vec<Layer> {
    let mut layers = Vec::new();
    if let Some(d) = tone_path(s.tone) {
        layers.push(Layer { role: Role::Tone, d, width: TONE_SW });
    }
    let has_final = s.nucleus.is_some() || s.coda.is_some() || s.medial.is_some();
    match (s.initial, has_final) {
        (Some(c), true) => {
            // ⿰: initial on the left like a radical, the final stacked on the right
            let left = Rect::new(4.0, BODY_TOP, 33.0, BODY_H);
            layers.push(layer(Role::Initial, consonant_path(c, left, aspect::INITIAL_SIDE)));
            let right = Rect::new(43.0, BODY_TOP, 53.0, BODY_H);
            final_layers(s, right, true, &mut layers);
        }
        (Some(c), false) => {
            // zhi chi shi ri zi ci si (and 儿化 on them): the initial alone
            if s.rhotic {
                let top = Rect::new(14.0, BODY_TOP, 72.0, 50.0);
                layers.push(layer(Role::Initial, consonant_path(c, top, (0.9, 1.6))));
                let bottom = Rect::new(14.0, BODY_TOP + 55.0, 72.0, 21.0);
                layers.push(layer(Role::Rhotic, rhotic_path(bottom, aspect::FRAME_CODA)));
            } else {
                let body = Rect::new(12.0, BODY_TOP + 1.0, 76.0, BODY_H - 2.0);
                layers.push(layer(Role::Initial, consonant_path(c, body, aspect::INITIAL_ALONE)));
            }
        }
        (None, true) => {
            let body = Rect::new(8.0, BODY_TOP, 84.0, BODY_H);
            final_layers(s, body, false, &mut layers);
        }
        (None, false) => {}
    }
    layers
}

enum FPart {
    Medial(Vowel),
    Nucleus(Vowel),
    Coda,
}

/// ⿱ stack: medial on top (flattened), vowel in the middle, coda at the bottom.
fn final_layers(s: &Syllable, rect: Rect, beside_initial: bool, layers: &mut Vec<Layer>) {
    let mut parts = Vec::new();
    if let Some(m) = s.medial {
        parts.push((FPart::Medial(m), if m == Vowel::Yu { 0.42 } else { 0.34 }));
    }
    if let Some(n) = s.nucleus {
        parts.push((FPart::Nucleus(n), 0.62));
    }
    if s.coda.is_some() || s.rhotic {
        let w = if s.coda.is_some() && s.rhotic { 0.4 } else { 0.32 };
        parts.push((FPart::Coda, w));
    }
    let gap = 5.5;
    let n = parts.len() as f32;
    let total: f32 = parts.iter().map(|p| p.1).sum();
    let avail = rect.h - gap * (n - 1.0);
    let mut y = rect.y;
    let alone = parts.len() == 1;
    for (part, weight) in parts {
        let h = if alone { rect.h } else { avail * weight / total };
        let r = Rect::new(rect.x, y, rect.w, h);
        y += h + gap;
        match part {
            FPart::Medial(v) => layers.push(layer(Role::Medial, vowel_path(v, r, glide_aspect(v)))),
            FPart::Nucleus(v) => {
                let asp = if !alone {
                    aspect::NUCLEUS_STACKED
                } else if beside_initial {
                    aspect::NUCLEUS_SIDE
                } else {
                    aspect::NUCLEUS_ALONE
                };
                layers.push(layer(Role::Nucleus, vowel_path(v, r, asp)));
            }
            FPart::Coda => coda_layers(s, r, alone, layers),
        }
    }
}

fn coda_layers(s: &Syllable, r: Rect, alone: bool, layers: &mut Vec<Layer>) {
    // With no vowel (m, n, ng, hm, hng) the nasal is syllabic and keeps its
    // nasal mark; after a vowel it is reduced to the bare frame.
    let syllabic = s.nucleus.is_none();
    let draw_coda = |c: Coda, r: Rect, asp: (f32, f32)| -> String {
        match (c.vowel(), c.consonant()) {
            (Some(v), _) => vowel_path(v, r, asp),
            (_, Some(k)) if syllabic => consonant_path(k, r, asp),
            (_, Some(k)) => consonant_path(coda_frame(k), r, asp),
            _ => String::new(),
        }
    };
    let coda_asp = |c: Coda| {
        if alone {
            aspect::INITIAL_ALONE
        } else if let Some(v) = c.vowel() {
            glide_aspect(v)
        } else {
            aspect::FRAME_CODA
        }
    };
    match (s.coda, s.rhotic) {
        (Some(c), true) => {
            // both: side by side, coda first
            let w = (r.w - 5.0) / 2.0;
            let left = Rect::new(r.x, r.y, w, r.h);
            let right = Rect::new(r.x + w + 5.0, r.y, w, r.h);
            layers.push(layer(Role::Coda, draw_coda(c, left, aspect::SPLIT)));
            layers.push(layer(Role::Rhotic, rhotic_path(right, aspect::SPLIT)));
        }
        (Some(c), false) => layers.push(layer(Role::Coda, draw_coda(c, r, coda_asp(c)))),
        (None, true) => layers.push(layer(Role::Rhotic, rhotic_path(r, aspect::FRAME_CODA))),
        (None, false) => {}
    }
}

// ---------------------------------------------------------------------------
// Single components, for the reference key
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Part {
    Consonant(Consonant),
    Vowel(Vowel),
    /// flattened glide form (medial / coda) of i, u, ü
    Glide(Vowel),
    /// a coda as written after a vowel (bare frame or flattened vowel)
    Coda(Coda),
    Rhotic,
    Tone(Tone),
}

pub fn part_layers(p: Part) -> Vec<Layer> {
    match p {
        Part::Consonant(c) => vec![layer(
            Role::Initial,
            consonant_path(c, Rect::new(16.0, 12.0, 68.0, 76.0), aspect::INITIAL_ALONE),
        )],
        Part::Vowel(v) => vec![layer(
            Role::Nucleus,
            vowel_path(v, Rect::new(12.0, 12.0, 76.0, 76.0), aspect::NUCLEUS_ALONE),
        )],
        Part::Glide(v) => vec![layer(
            Role::Medial,
            vowel_path(v, Rect::new(8.0, 34.0, 84.0, 32.0), glide_aspect(v)),
        )],
        Part::Coda(c) => {
            let r = Rect::new(8.0, 30.0, 84.0, 40.0);
            let d = match (c.vowel(), c.consonant()) {
                (Some(v), _) => vowel_path(v, r, glide_aspect(v)),
                (_, Some(k)) => consonant_path(coda_frame(k), r, aspect::FRAME_CODA),
                _ => String::new(),
            };
            vec![layer(Role::Coda, d)]
        }
        Part::Rhotic => vec![layer(
            Role::Rhotic,
            rhotic_path(Rect::new(10.0, 30.0, 80.0, 40.0), aspect::FRAME_CODA),
        )],
        Part::Tone(t) => tone_path(t)
            .map(|d| {
                // scale the mark up so it reads on its own
                let scaled = scale_path(&d, 50.0, 9.0, 3.4, 50.0, 50.0);
                vec![Layer { role: Role::Tone, d: scaled, width: SW }]
            })
            .unwrap_or_default(),
    }
}

/// Scale an absolute M/L path around (cx, cy) and move it to (tx, ty).
fn scale_path(d: &str, cx: f32, cy: f32, k: f32, tx: f32, ty: f32) -> String {
    let mut out = String::new();
    let mut num = String::new();
    let mut nums: Vec<f32> = Vec::new();
    let mut cmd = ' ';
    let flush = |cmd: char, nums: &mut Vec<f32>, out: &mut String| {
        if cmd == ' ' {
            return;
        }
        out.push(cmd);
        for (i, v) in nums.iter().enumerate() {
            let t = if i % 2 == 0 { (v - cx) * k + tx } else { (v - cy) * k + ty };
            if i > 0 {
                out.push(' ');
            }
            let _ = write!(out, "{t:.1}");
        }
        nums.clear();
    };
    for ch in d.chars() {
        if ch.is_ascii_alphabetic() {
            if !num.is_empty() {
                nums.push(num.parse().unwrap_or(0.0));
                num.clear();
            }
            flush(cmd, &mut nums, &mut out);
            cmd = ch;
        } else if ch == ' ' {
            if !num.is_empty() {
                nums.push(num.parse().unwrap_or(0.0));
                num.clear();
            }
        } else {
            num.push(ch);
        }
    }
    if !num.is_empty() {
        nums.push(num.parse().unwrap_or(0.0));
    }
    flush(cmd, &mut nums, &mut out);
    out
}

// ---------------------------------------------------------------------------
// Human-readable breakdown
// ---------------------------------------------------------------------------

pub struct PartInfo {
    pub role: Role,
    pub sound: String,
    pub component: String,
    pub note: String,
}

pub fn describe(s: &Syllable) -> Vec<PartInfo> {
    let mut v = Vec::new();
    let consonant_component = |c: Consonant| match c.manner {
        Manner::Plain => c.place.frame().to_string(),
        m => format!("{} + {}", c.place.frame(), m.mark()),
    };
    if let Some(c) = s.initial {
        v.push(PartInfo {
            role: Role::Initial,
            sound: c.pinyin().to_string(),
            component: consonant_component(c),
            note: format!("{} frame, {} mark", c.place.name(), c.manner.name().to_lowercase()),
        });
    }
    if let Some(m) = s.medial {
        v.push(PartInfo {
            role: Role::Medial,
            sound: m.pinyin().to_string(),
            component: format!("{} (flattened)", m.component()),
            note: "glide into the vowel, written on top".into(),
        });
    }
    match s.nucleus {
        Some(n) => v.push(PartInfo {
            role: Role::Nucleus,
            sound: n.pinyin().to_string(),
            component: n.component().to_string(),
            note: "main vowel".into(),
        }),
        None if s.initial.is_some() && s.coda.is_none() => v.push(PartInfo {
            role: Role::Nucleus,
            sound: "-i".into(),
            component: "(none)".into(),
            note: "buzzing 'empty' vowel: the initial is written alone".into(),
        }),
        None => {}
    }
    if let Some(c) = s.coda {
        let (component, note) = match (c.vowel(), c.consonant()) {
            (Some(v), _) => (format!("{} (flattened)", v.component()), "glide out of the vowel"),
            (_, Some(k)) if s.nucleus.is_none() => {
                (consonant_component(k), "syllabic nasal: the full consonant")
            }
            (_, Some(k)) => (
                k.place.frame().to_string(),
                "bare frame: after a vowel only a nasal can follow, so no mark is needed",
            ),
            _ => (String::new(), ""),
        };
        let sound = if c == Coda::U && s.nucleus == Some(Vowel::A) { "o (u)" } else { c.pinyin() };
        v.push(PartInfo { role: Role::Coda, sound: sound.into(), component, note: note.into() });
    }
    if s.rhotic {
        v.push(PartInfo {
            role: Role::Rhotic,
            sound: "r".into(),
            component: "乚".into(),
            note: "bare retroflex frame: r-colouring (儿化 / er)".into(),
        });
    }
    v.push(PartInfo {
        role: Role::Tone,
        sound: match s.tone {
            Tone::Neutral => "neutral".into(),
            t => format!("tone {}", t.number()),
        },
        component: s.tone.pinyin_mark().into(),
        note: s.tone.stroke_name().into(),
    });
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_finals_renders() {
        for (key, ..) in crate::phonology::FINALS {
            let s = Syllable::from_pinyin(&format!("d{key}")).or_else(|| Syllable::from_pinyin(key));
            if let Some(s) = s {
                let layers = syllable_layers(&s);
                assert!(!layers.is_empty(), "{key}");
                for l in layers {
                    assert!(!l.d.contains("NaN"), "{key}: {}", l.d);
                }
            }
        }
    }

    #[test]
    fn scale_path_works() {
        assert_eq!(scale_path("M40 9.5L60 9.5", 50.0, 9.0, 2.0, 50.0, 50.0), "M30.0 51.0L70.0 51.0");
    }
}
