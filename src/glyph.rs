//! Stroke shapes for every component and the block layout that assembles
//! them into one square glyph per syllable.
//!
//! A block has at most four parts: tone, initial, medial and rhyme. A rhyme
//! (韵) is one symbol: its vowel plus at most one ending stroke.
//!
//! All geometry lives in a 100 × 100 em box. Components are drawn as
//! centre-line strokes (round caps/joins, constant width, like a 圆体 /
//! rounded-Hei typeface) in a unit box, then mapped into the rectangle the
//! layout assigns them. Because the stroke width is applied after mapping,
//! squashing a component into a flat slot keeps every stroke the same weight,
//! just as 土 keeps its weight whether it is the full character or the
//! bottom of 坐.

use crate::phonology::{Coda, Consonant, Manner, Place, Syllable, Tone, Vowel, R_IDEA, R_STROKE};
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
// Vowels: one or two strokes each
// ---------------------------------------------------------------------------

/// 八 a: two strokes held well apart (人 ê is the same pair, closed at the top)
const BA: Strokes = &[
    &[M(0.34, 0.04), Q(0.32, 0.62, 0.02, 0.96)],
    &[M(0.66, 0.08), Q(0.7, 0.62, 0.98, 0.94)],
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

/// 人 ê: the front mid vowel (ie, üe, and the start of ei)
const REN: Strokes = &[
    &[M(0.54, 0.02), Q(0.48, 0.6, 0.02, 0.96)],
    &[M(0.51, 0.3), Q(0.66, 0.8, 0.98, 0.96)],
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
        Vowel::Ehat => REN,
        Vowel::I => YI,
        Vowel::U => WU,
        Vowel::Yu => YU,
    }
}

// ---------------------------------------------------------------------------
// Ending strokes: what turns a vowel into a rhyme (a → ai ao an ang, o → ou …)
// Each is exactly one stroke, written along the foot of the vowel.
// ---------------------------------------------------------------------------

/// -i: 提, a rising stroke, the tongue rising toward i
const END_I: Strokes = &[&[M(0.2, 0.94), L(0.8, 0.06)]];
/// -u: 卧钩, a bowl, the lips rounding toward u
const END_U: Strokes = &[&[M(0.03, 0.08), Q(0.3, 1.0, 0.8, 0.9), L(0.97, 0.36)]];
/// -n: 横, a level bar, the tongue tip sealing the mouth
const END_N: Strokes = &[&[M(0.02, 0.5), L(0.98, 0.5)]];
/// -ng: 横折钩, the same bar closed at the back
const END_NG: Strokes = &[&[M(0.02, 0.08), L(0.96, 0.08), L(0.96, 0.94), L(0.8, 0.8)]];
/// -r: 竖弯钩, the retroflex curl (儿化, er)
const END_R: Strokes = &[&[
    M(0.04, 0.0),
    L(0.04, 0.5),
    Q(0.04, 0.94, 0.3, 0.94),
    L(0.86, 0.94),
    L(0.97, 0.58),
]];

/// (strokes, min aspect, max aspect); `None` is the r ending.
fn ending(c: Option<Coda>) -> (Strokes, f32, f32) {
    match c {
        Some(Coda::I) => (END_I, 0.9, 1.8),
        Some(Coda::U) => (END_U, 1.8, 3.4),
        Some(Coda::N) => (END_N, 2.0, 99.0),
        Some(Coda::Ng) => (END_NG, 1.8, 3.4),
        Some(Coda::M) => (END_N, 2.0, 99.0),
        None => (END_R, 1.6, 3.2),
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
    Rhyme,
}

impl Role {
    pub fn class(self) -> &'static str {
        match self {
            Role::Tone => "r-tone",
            Role::Initial => "r-initial",
            Role::Medial => "r-medial",
            Role::Rhyme => "r-rhyme",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Role::Tone => "Tone",
            Role::Initial => "Initial",
            Role::Medial => "Medial",
            Role::Rhyme => "Rhyme",
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
    pub const GLIDE: (f32, f32) = (1.6, 3.4);
    /// a rhyme under a medial
    pub const RHYME_STACKED: (f32, f32) = (0.7, 1.7);
    /// a rhyme beside an initial, no medial
    pub const RHYME_SIDE: (f32, f32) = (0.5, 1.0);
    /// a rhyme filling the block
    pub const RHYME_ALONE: (f32, f32) = (0.7, 1.06);
    /// the vowel inside a rhyme that has an ending
    pub const RHYME_VOWEL: (f32, f32) = (0.6, 2.2);
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

/// How flat a medial may be drawn: crossed strokes (乂, 义) stop reading as a
/// cross when squashed too far, 乙 survives more.
fn glide_aspect(v: Vowel) -> (f32, f32) {
    match v {
        Vowel::I => aspect::GLIDE,
        _ => (aspect::GLIDE.0, 2.3),
    }
}

/// Draw endings (coda and/or r) side by side into a flat band.
fn endings_path(endings: &[Option<Coda>], band: Rect, out: &mut String) {
    let gap = 5.0;
    let n = endings.len() as f32;
    let w = (band.w - gap * (n - 1.0)) / n;
    for (k, e) in endings.iter().enumerate() {
        let (strokes, lo, hi) = ending(*e);
        let r = Rect::new(band.x + k as f32 * (w + gap), band.y, w, band.h);
        emit(strokes, r.fit(lo, hi).inset(SW / 2.0), out);
    }
}

/// One rhyme symbol: the vowel, with its ending stroke tucked under it.
fn rhyme_path(nucleus: Vowel, coda: Option<Coda>, rhotic: bool, r: Rect, asp: (f32, f32)) -> String {
    let mut out = String::new();
    let r = r.fit(asp.0, asp.1);
    let mut endings = Vec::new();
    if coda.is_some() {
        endings.push(coda);
    }
    if rhotic {
        endings.push(None);
    }
    if endings.is_empty() {
        emit(vowel_strokes(nucleus), r.inset(SW / 2.0), &mut out);
        return out;
    }
    // The ending sits close under the vowel (closer than any gap between
    // parts), so vowel + ending read as one symbol.
    let band_h = r.h * 0.3;
    let gap = r.h * 0.03;
    let top = Rect::new(r.x, r.y, r.w, r.h - band_h - gap);
    let vowel = top.fit(aspect::RHYME_VOWEL.0, aspect::RHYME_VOWEL.1);
    emit(vowel_strokes(nucleus), vowel.inset(SW / 2.0), &mut out);
    let band = Rect::new(vowel.x, r.y + r.h - band_h, vowel.w, band_h);
    endings_path(&endings, band, &mut out);
    out
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
            // initial on the left like a radical, the final on the right
            let left = Rect::new(4.0, BODY_TOP, 33.0, BODY_H);
            layers.push(layer(Role::Initial, consonant_path(c, left, aspect::INITIAL_SIDE)));
            let right = Rect::new(43.0, BODY_TOP, 53.0, BODY_H);
            final_layers(s, right, true, &mut layers);
        }
        (Some(c), false) => {
            // zhi chi shi ri zi ci si: the initial alone (plus the r curl for 儿化)
            if s.rhotic {
                let top = Rect::new(14.0, BODY_TOP, 72.0, 52.0);
                layers.push(layer(Role::Initial, consonant_path(c, top, (0.9, 1.6))));
                let mut d = String::new();
                endings_path(&[None], Rect::new(26.0, BODY_TOP + 57.0, 48.0, 19.0), &mut d);
                layers.push(layer(Role::Rhyme, d));
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

/// The final: an optional flattened medial on top, then one rhyme symbol.
fn final_layers(s: &Syllable, rect: Rect, beside_initial: bool, layers: &mut Vec<Layer>) {
    let Some(nucleus) = s.nucleus else {
        // syllabic nasal (m, n, ng, hm, hng): the full nasal consonant
        if let Some(k) = s.coda.and_then(|c| c.consonant()) {
            let asp = if beside_initial { aspect::INITIAL_SIDE } else { aspect::INITIAL_ALONE };
            layers.push(layer(Role::Rhyme, consonant_path(k, rect, asp)));
        }
        return;
    };
    let has_ending = s.coda.is_some() || s.rhotic;
    match s.medial {
        Some(m) => {
            let mw = if m == Vowel::Yu { 0.42 } else { 0.34 };
            let rw = if has_ending { 0.95 } else { 0.62 };
            let gap = 6.0;
            let avail = rect.h - gap;
            let mh = avail * mw / (mw + rw);
            let mr = Rect::new(rect.x, rect.y, rect.w, mh);
            layers.push(layer(Role::Medial, vowel_path(m, mr, glide_aspect(m))));
            let rr = Rect::new(rect.x, rect.y + mh + gap, rect.w, avail - mh);
            let d = rhyme_path(nucleus, s.coda, s.rhotic, rr, aspect::RHYME_STACKED);
            layers.push(layer(Role::Rhyme, d));
        }
        None => {
            let asp = if beside_initial { aspect::RHYME_SIDE } else { aspect::RHYME_ALONE };
            let d = rhyme_path(nucleus, s.coda, s.rhotic, rect, asp);
            layers.push(layer(Role::Rhyme, d));
        }
    }
}

// ---------------------------------------------------------------------------
// Single components, for the reference key
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Part {
    Consonant(Consonant),
    /// a vowel on its own, full size
    Vowel(Vowel),
    /// flattened medial form of i, u, ü
    Glide(Vowel),
    /// a whole rhyme symbol: vowel + ending (+ r)
    Rhyme(Vowel, Option<Coda>, bool),
    /// one ending stroke on its own; `None` is the r curl
    Ending(Option<Coda>),
    Tone(Tone),
}

pub fn part_layers(p: Part) -> Vec<Layer> {
    match p {
        Part::Consonant(c) => vec![layer(
            Role::Initial,
            consonant_path(c, Rect::new(16.0, 12.0, 68.0, 76.0), aspect::INITIAL_ALONE),
        )],
        Part::Vowel(v) => vec![layer(
            Role::Rhyme,
            vowel_path(v, Rect::new(12.0, 12.0, 76.0, 76.0), aspect::RHYME_ALONE),
        )],
        Part::Glide(v) => vec![layer(
            Role::Medial,
            vowel_path(v, Rect::new(8.0, 34.0, 84.0, 32.0), glide_aspect(v)),
        )],
        Part::Rhyme(v, c, r) => vec![layer(
            Role::Rhyme,
            rhyme_path(v, c, r, Rect::new(12.0, 12.0, 76.0, 76.0), aspect::RHYME_ALONE),
        )],
        Part::Ending(e) => {
            let mut d = String::new();
            endings_path(&[e], Rect::new(14.0, 32.0, 72.0, 36.0), &mut d);
            vec![layer(Role::Rhyme, d)]
        }
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

fn consonant_component(c: Consonant) -> String {
    match c.manner {
        Manner::Plain => c.place.frame().to_string(),
        m => format!("{} + {}", c.place.frame(), m.mark()),
    }
}

pub fn describe(s: &Syllable) -> Vec<PartInfo> {
    let mut v = Vec::new();
    if let Some(c) = s.initial {
        v.push(PartInfo {
            role: Role::Initial,
            sound: format!("{} {}", c.zhuyin(), c.pinyin()),
            component: consonant_component(c),
            note: format!("{} frame, {} mark", c.place.name(), c.manner.name().to_lowercase()),
        });
    }
    if let Some(m) = s.medial {
        v.push(PartInfo {
            role: Role::Medial,
            sound: format!("{} {}", m.zhuyin(), m.pinyin()),
            component: format!("{} (flattened)", m.component()),
            note: "glide into the rhyme, written on top".into(),
        });
    }
    match s.nucleus {
        Some(n) => {
            let mut component = n.component().to_string();
            let mut notes = Vec::new();
            if let Some(c) = s.coda {
                component.push_str(&format!(" + {}", c.stroke()));
                notes.push(c.idea());
            }
            if s.rhotic {
                component.push_str(&format!(" + {R_STROKE}"));
                notes.push(R_IDEA);
            }
            let note = if notes.is_empty() {
                format!("the vowel {} on its own", n.pinyin())
            } else {
                format!("one symbol: {} plus {}", n.pinyin(), notes.join("; "))
            };
            let sound = format!("{} {}", s.rhyme_zhuyin(), s.rhyme_name());
            v.push(PartInfo { role: Role::Rhyme, sound, component, note });
        }
        None => match s.coda.and_then(|c| c.consonant()) {
            Some(k) => v.push(PartInfo {
                role: Role::Rhyme,
                sound: k.pinyin().to_string(),
                component: consonant_component(k),
                note: "syllabic nasal: the full consonant".into(),
            }),
            None if s.initial.is_some() => v.push(PartInfo {
                role: Role::Rhyme,
                sound: if s.rhotic { "-ir".into() } else { "-i".into() },
                component: if s.rhotic { R_STROKE.into() } else { "(none)".into() },
                note: "buzzing 'empty' vowel: the initial is written alone".into(),
            }),
            None => {}
        },
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
    fn every_final_renders() {
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
    fn a_final_has_at_most_medial_and_rhyme() {
        let s = Syllable::from_pinyin("zhuang").unwrap();
        let roles: Vec<Role> = syllable_layers(&s).iter().map(|l| l.role).collect();
        assert_eq!(roles, [Role::Initial, Role::Medial, Role::Rhyme]);
    }

    #[test]
    fn scale_path_works() {
        assert_eq!(scale_path("M40 9.5L60 9.5", 50.0, 9.0, 2.0, 50.0, 50.0), "M30.0 51.0L70.0 51.0");
    }
}
