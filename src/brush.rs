//! A small brush simulator.
//!
//! Glyph components are designed as centre-lines. This module "writes" each
//! centre-line with a simulated brush and returns the filled outline:
//!
//! 1. **Path shaping**: straight strokes get a slight bow, existing curves get
//!    deeper, horizontals rise to the right (a y-shear, so verticals stay
//!    vertical), hooks flick out further, and a smooth seeded wobble imitates
//!    an unsteady hand.
//! 2. **Stroke reading**: every stroke is classified from its own geometry,
//!    the way a calligrapher would name it: a pressed 横/竖 end, a sweeping 撇,
//!    a rising 提, a 捺 with a foot, a 点, and whether it ends in a hook (钩).
//!    Sharp turns (折) split the stroke into pieces.
//! 3. **Pressure**: a width profile along the stroke: pressed entries and
//!    pauses (顿笔), heavier verticals than horizontals, a slimmer waist on long
//!    strokes, pressed corners, tapering tips, the swelling foot of a 捺 and
//!    the teardrop of a 点.
//! 4. **Outline**: each piece is offset to both sides by half its width and
//!    capped with round ends, which also gives round joins at the corners. The
//!    pieces are emitted as sub-paths with the same winding, so with the
//!    nonzero fill rule they merge into one shape.
//!
//! With every knob at zero the result is the plain geometric rendering:
//! constant width, round caps and round joins.

use crate::glyph::{Layer, Seg, Stroke};
use std::f32::consts::PI;
use std::fmt::Write;

/// The tuning knobs. All are 0–1 except `weight`, a multiplier on the nominal
/// stroke width.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Brush {
    /// overall stroke thickness
    pub weight: f32,
    /// thick–thin contrast: slimmer waists, heavier verticals, swelling 捺 feet
    pub pressure: f32,
    /// pointed tips on 撇 提 and hooks, thin entries on 点 and 捺
    pub taper: f32,
    /// 顿笔: pressed heads and tails on 横 and 竖, pressed corners on 折
    pub pause: f32,
    /// arcs: bowed straight strokes, deeper curves, horizontals rising rightward
    pub arc: f32,
    /// hooks (钩) flick out further
    pub hook: f32,
    /// an unsteady hand
    pub wobble: f32,
}

impl Brush {
    pub const GEOMETRIC: Brush = Brush {
        weight: 1.0,
        pressure: 0.0,
        taper: 0.0,
        pause: 0.0,
        arc: 0.0,
        hook: 0.0,
        wobble: 0.0,
    };

    pub const PEN: Brush = Brush {
        weight: 0.92,
        pressure: 0.3,
        taper: 0.6,
        pause: 0.25,
        arc: 0.45,
        hook: 0.45,
        wobble: 0.3,
    };

    pub const BRUSH: Brush = Brush {
        weight: 1.08,
        pressure: 0.85,
        taper: 0.95,
        pause: 0.8,
        arc: 0.7,
        hook: 0.85,
        wobble: 0.4,
    };

    /// Blend from the geometric rendering (0) to the full brush (1).
    pub fn hand(t: f32) -> Brush {
        let t = t.clamp(0.0, 1.0);
        let (a, b) = (Brush::GEOMETRIC, Brush::BRUSH);
        let mix = |x: f32, y: f32| x + (y - x) * t;
        Brush {
            weight: mix(a.weight, b.weight),
            pressure: mix(a.pressure, b.pressure),
            taper: mix(a.taper, b.taper),
            pause: mix(a.pause, b.pause),
            arc: mix(a.arc, b.arc),
            hook: mix(a.hook, b.hook),
            wobble: mix(a.wobble, b.wobble),
        }
    }

    fn shapes_path(&self) -> bool {
        self.arc > 0.0 || self.hook > 0.0 || self.wobble > 0.0
    }

    fn varies_width(&self) -> bool {
        self.pressure > 0.0 || self.taper > 0.0 || self.pause > 0.0
    }
}

impl Default for Brush {
    fn default() -> Self {
        Brush::GEOMETRIC
    }
}

// ---------------------------------------------------------------------------
// Small geometry helpers
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq)]
struct P {
    x: f32,
    y: f32,
}

impl P {
    fn new(x: f32, y: f32) -> P {
        P { x, y }
    }
    fn sub(self, o: P) -> P {
        P::new(self.x - o.x, self.y - o.y)
    }
    fn add(self, o: P) -> P {
        P::new(self.x + o.x, self.y + o.y)
    }
    fn mul(self, k: f32) -> P {
        P::new(self.x * k, self.y * k)
    }
    fn len(self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
    fn norm(self) -> P {
        let l = self.len();
        if l < 1e-6 {
            P::new(1.0, 0.0)
        } else {
            self.mul(1.0 / l)
        }
    }
    fn lerp(self, o: P, t: f32) -> P {
        self.add(o.sub(self).mul(t))
    }
}

fn quad(a: P, c: P, b: P, t: f32) -> P {
    let u = 1.0 - t;
    P::new(
        u * u * a.x + 2.0 * u * t * c.x + t * t * b.x,
        u * u * a.y + 2.0 * u * t * c.y + t * t * b.y,
    )
}

fn smooth01(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

/// 1 at d = 0, easing to 0 at d = len.
fn fall(d: f32, len: f32) -> f32 {
    if d < 0.0 {
        return 0.0;
    }
    1.0 - smooth01(d / len)
}

fn bell(d: f32, sigma: f32) -> f32 {
    (-(d / sigma) * (d / sigma)).exp()
}

fn angle_between(a: P, b: P) -> f32 {
    let (a, b) = (a.norm(), b.norm());
    (a.x * b.x + a.y * b.y).clamp(-1.0, 1.0).acos()
}

/// A tiny deterministic hash, so the wobble of a stroke is stable.
fn hash(mut h: u32, v: f32) -> u32 {
    h ^= v.to_bits();
    h = h.wrapping_mul(0x9E37_79B1);
    h ^= h >> 15;
    h.wrapping_mul(0x85EB_CA6B)
}

fn unit(h: u32) -> f32 {
    (h >> 8) as f32 / (1u32 << 24) as f32
}

// ---------------------------------------------------------------------------
// 1. Path shaping and sampling
// ---------------------------------------------------------------------------

struct Sampled {
    pts: Vec<P>,
    /// indices into `pts` where the stroke turns sharply (折)
    corners: Vec<usize>,
}

/// Turn the stroke's segments into a dense polyline, bending it on the way.
fn sample(stroke: &Stroke, brush: &Brush) -> Sampled {
    let dense = brush.shapes_path() || brush.varies_width();
    let step = 2.2;
    let mut pts: Vec<P> = Vec::new();
    let mut corners = Vec::new();
    let mut cur = P::new(0.0, 0.0);
    let mut last_dir: Option<P> = None;

    for seg in stroke {
        let (ctrl, end) = match *seg {
            Seg::M(x, y) => {
                cur = P::new(x, y);
                pts.push(cur);
                continue;
            }
            Seg::L(x, y) => {
                let end = P::new(x, y);
                (bow(cur, end, brush.arc), end)
            }
            Seg::Q(cx, cy, x, y) => {
                let end = P::new(x, y);
                let c = P::new(cx, cy);
                let mid = cur.lerp(end, 0.5);
                // deepen designed curves a little
                (Some(mid.add(c.sub(mid).mul(1.0 + 0.35 * brush.arc))), end)
            }
        };
        let first_dir = match ctrl {
            Some(c) => c.sub(cur),
            None => end.sub(cur),
        };
        if let Some(d) = last_dir {
            if angle_between(d, first_dir) > 25f32.to_radians() {
                corners.push(pts.len() - 1);
            }
        }
        let chord = end.sub(cur).len();
        match ctrl {
            None => {
                let n = if dense { (chord / step).ceil().max(1.0) as usize } else { 1 };
                for i in 1..=n {
                    pts.push(cur.lerp(end, i as f32 / n as f32));
                }
                last_dir = Some(end.sub(cur));
            }
            Some(c) => {
                let approx = cur.sub(c).len() + c.sub(end).len();
                let n = (approx / step).ceil().max(4.0) as usize;
                for i in 1..=n {
                    pts.push(quad(cur, c, end, i as f32 / n as f32));
                }
                last_dir = Some(end.sub(c));
            }
        }
        cur = end;
    }
    Sampled { pts, corners }
}

/// Straight horizontals bow upward and diagonals bow outward, slightly.
fn bow(a: P, b: P, arc: f32) -> Option<P> {
    if arc <= 0.0 {
        return None;
    }
    let d = b.sub(a);
    let len = d.len();
    if len < 12.0 {
        return None;
    }
    let mid = a.lerp(b, 0.5);
    let (ax, ay) = (d.x.abs(), d.y.abs());
    if ax > 1.6 * ay {
        // 横: a gentle upward arch
        Some(mid.add(P::new(0.0, -0.035 * arc * len)))
    } else if ay > 1.6 * ax {
        // 竖: stays straight and confident
        None
    } else {
        // diagonals: 撇 bulge to the lower right, 捺 to the lower left
        let n = P::new(d.y, -d.x).norm();
        let sign = if d.x < 0.0 { 1.0 } else { -1.0 };
        Some(mid.add(n.mul(sign * 0.05 * arc * len)))
    }
}

// ---------------------------------------------------------------------------
// 2. Reading the stroke
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq)]
enum End {
    /// 横, 竖: the brush stops and lifts
    Blunt,
    /// 撇: sweeps out to a point
    Sweep,
    /// 提: flicks up to a point
    Rise,
    /// 捺: swells into a foot, then leaves sideways
    Foot,
    /// 点: a teardrop
    Dot,
}

struct Reading {
    /// arc length at every point
    s: Vec<f32>,
    total: f32,
    /// arc length at every corner
    corner_s: Vec<f32>,
    /// arc length where the hook starts, if the stroke ends in one
    hook_from: Option<f32>,
    /// arc length where the main body of the stroke ends
    body_end: f32,
    end: End,
    thin_start: bool,
}

/// Does the stroke end in a hook (钩)? True when its last piece is a short,
/// sharp turn compared with the piece before it. Returns the corner index.
fn hook_corner(pts: &[P], corners: &[usize]) -> Option<usize> {
    let &c = corners.last()?;
    let piece_len = |a: usize, b: usize| -> f32 { (a..b).map(|i| pts[i + 1].sub(pts[i]).len()).sum() };
    let tail = piece_len(c, pts.len() - 1);
    let prev_start = corners.len().checked_sub(2).map(|k| corners[k]).unwrap_or(0);
    let prev = piece_len(prev_start, c);
    (tail < 20.0 && tail < 0.45 * prev).then_some(c)
}

fn read(pts: &[P], corners: &[usize], hook: Option<usize>) -> Reading {
    let mut s = Vec::with_capacity(pts.len());
    let mut acc = 0.0;
    for (i, p) in pts.iter().enumerate() {
        if i > 0 {
            acc += p.sub(pts[i - 1]).len();
        }
        s.push(acc);
    }
    let total = acc.max(0.01);
    let corner_s: Vec<f32> = corners.iter().map(|&i| s[i]).collect();

    let hook_from = hook.map(|c| s[c]);
    let body_end = hook_from.unwrap_or(total);

    // the direction in which the main body leaves the paper
    let end_dir = {
        let mut i = pts.len() - 1;
        if let Some(h) = hook_from {
            while i > 0 && s[i] > h + 1e-3 {
                i -= 1;
            }
        }
        let mut j = i;
        while j > 0 && s[i] - s[j] < 4.0 {
            j -= 1;
        }
        pts[i].sub(pts[j]).norm()
    };
    let single = corners.is_empty();
    let end = if hook_from.is_some() {
        End::Blunt
    } else if end_dir.x > 0.3 && end_dir.y < -0.3 {
        End::Rise
    } else if end_dir.x < -0.3 && end_dir.y > 0.3 {
        End::Sweep
    } else if single && total < 17.0 {
        End::Dot
    } else if single && end_dir.x > 0.35 && end_dir.y > 0.25 && total > 22.0 {
        End::Foot
    } else {
        End::Blunt
    };
    let thin_start = matches!(end, End::Dot | End::Foot);
    Reading { s, total, corner_s, hook_from, body_end, end, thin_start }
}

// ---------------------------------------------------------------------------
// 3. Pressure
// ---------------------------------------------------------------------------

fn width_factor(r: &Reading, i: usize, tangent: P, b: &Brush) -> f32 {
    let s = r.s[i];
    let total = r.total;
    let mut f = 1.0;

    // verticals carry more ink than horizontals
    f *= 1.0 + 0.14 * b.pressure * (tangent.y.abs() - tangent.x.abs());

    // entry
    if r.thin_start {
        let ramp = smooth01(s / (0.5 * total));
        f *= 1.0 + b.taper * ((0.3 + 0.7 * ramp) - 1.0);
    } else {
        f *= 1.0 + 0.24 * b.pause * fall(s, 6.0);
    }

    // a slimmer waist on long pieces
    let mut piece_start = 0.0;
    for &edge in r.corner_s.iter().chain(std::iter::once(&total)) {
        let len = edge - piece_start;
        if len > 18.0 && s >= piece_start && s <= edge {
            let u = (s - piece_start) / len;
            f *= 1.0 - 0.17 * b.pressure * (PI * u).sin().powi(2);
        }
        piece_start = edge;
    }

    // pressed corners (折)
    for &c in &r.corner_s {
        f *= 1.0 + 0.26 * b.pause * bell(s - c, 4.5);
    }

    // exit
    let body = r.body_end;
    match r.end {
        End::Blunt => f *= 1.0 + 0.16 * b.pause * fall(body - s, 5.0),
        End::Sweep | End::Rise => {
            // a lone 撇 / 提 thins from early on; after a turn (横撇) the
            // sweep starts thick at the corner and thins along its own piece
            let from = match r.corner_s.last() {
                Some(&c) => c + 0.1 * (body - c),
                None => body * 0.4,
            };
            f *= 1.0 - 0.92 * b.taper * smooth01((s - from) / (body - from));
            if r.end == End::Sweep {
                f *= 1.0 + 0.12 * b.pressure * (1.0 - s / body);
            }
        }
        End::Foot => {
            f *= 1.0 + 0.5 * b.pressure * bell(s - 0.8 * total, 0.13 * total);
            f *= 1.0 - 0.93 * b.taper * smooth01((s - 0.86 * total) / (0.14 * total));
        }
        End::Dot => {
            let k = b.taper.max(b.pressure);
            f *= 1.0 + k * ((0.4 + 0.85 * s / total) - 1.0);
        }
    }

    // a hook leaves the paper as a sharp flick
    if let Some(h) = r.hook_from {
        if s > h {
            f *= 1.0 - 0.94 * b.taper * smooth01((s - h) / (total - h));
        }
    }

    f.max(0.06)
}

// ---------------------------------------------------------------------------
// 4. Outline
// ---------------------------------------------------------------------------

/// The filled outline of one stroke, as SVG path data.
pub fn stroke_outline(stroke: &Stroke, width: f32, brush: &Brush, seed: u32, out: &mut String) {
    let Sampled { mut pts, corners } = sample(stroke, brush);
    if pts.len() < 2 {
        if let Some(&p) = pts.first() {
            circle(p, width * brush.weight / 2.0, out);
        }
        return;
    }

    // hooks flick out further
    let hook = hook_corner(&pts, &corners);
    if let (Some(c), true) = (hook, brush.hook > 0.0) {
        let k = 1.0 + 0.55 * brush.hook;
        let start = pts[c];
        for p in pts.iter_mut().skip(c + 1) {
            *p = start.add(p.sub(start).mul(k));
        }
    }

    // an unsteady hand: smooth sideways drift, different for every stroke
    if brush.wobble > 0.0 {
        let h1 = hash(seed, pts[0].x * 7.0 + pts[0].y);
        let h2 = hash(h1, pts[pts.len() - 1].x);
        let (ph1, ph2) = (unit(h1) * 2.0 * PI, unit(h2) * 2.0 * PI);
        let (l1, l2) = (28.0 + 20.0 * unit(h2), 9.0 + 6.0 * unit(h1));
        let amp = 1.1 * brush.wobble;
        let mut s = 0.0;
        let orig = pts.clone();
        for i in 0..pts.len() {
            if i > 0 {
                s += orig[i].sub(orig[i - 1]).len();
            }
            let t = tangent(&orig, i, &corners);
            let n = P::new(-t.y, t.x);
            let off = amp * (0.65 * (2.0 * PI * s / l1 + ph1).sin() + 0.35 * (2.0 * PI * s / l2 + ph2).sin());
            pts[i] = orig[i].add(n.mul(off));
        }
    }

    // horizontals rise to the right: a shear that keeps verticals vertical
    if brush.arc > 0.0 {
        let k = 0.05 * brush.arc;
        for p in pts.iter_mut() {
            p.y -= k * (p.x - 50.0);
        }
    }

    let reading = read(&pts, &corners, hook);
    let base = width * brush.weight;
    let widths: Vec<f32> = (0..pts.len())
        .map(|i| {
            if brush.varies_width() {
                base * width_factor(&reading, i, tangent(&pts, i, &corners), brush)
            } else {
                base
            }
        })
        .collect();

    // one sub-path per piece between corners
    let mut bounds = vec![0];
    bounds.extend(corners.iter().copied());
    bounds.push(pts.len() - 1);
    for w in bounds.windows(2) {
        piece_outline(&pts[w[0]..=w[1]], &widths[w[0]..=w[1]], out);
    }
}

fn tangent(pts: &[P], i: usize, corners: &[usize]) -> P {
    let n = pts.len();
    let is_corner = corners.contains(&i);
    let prev = if i > 0 { Some(pts[i].sub(pts[i - 1])) } else { None };
    let next = if i + 1 < n { Some(pts[i + 1].sub(pts[i])) } else { None };
    match (prev, next) {
        (Some(a), Some(b)) if !is_corner => a.norm().add(b.norm()).norm(),
        (Some(a), _) => a.norm(),
        (_, Some(b)) => b.norm(),
        _ => P::new(1.0, 0.0),
    }
}

fn piece_outline(pts: &[P], widths: &[f32], out: &mut String) {
    let n = pts.len();
    // tangents within the piece only, so corners don't bend the offsets
    let tan: Vec<P> = (0..n)
        .map(|i| {
            let a = if i > 0 { Some(pts[i].sub(pts[i - 1])) } else { None };
            let b = if i + 1 < n { Some(pts[i + 1].sub(pts[i])) } else { None };
            match (a, b) {
                (Some(a), Some(b)) => a.norm().add(b.norm()).norm(),
                (Some(a), None) => a.norm(),
                (None, Some(b)) => b.norm(),
                _ => P::new(1.0, 0.0),
            }
        })
        .collect();
    let side = |i: usize, sign: f32| {
        let t = tan[i];
        let nrm = P::new(-t.y, t.x);
        pts[i].add(nrm.mul(sign * widths[i] / 2.0))
    };
    let f = |v: f32| (v * 10.0).round() / 10.0;
    let a = side(0, 1.0);
    let _ = write!(out, "M{} {}", f(a.x), f(a.y));
    for i in 1..n {
        let p = side(i, 1.0);
        let _ = write!(out, "L{} {}", f(p.x), f(p.y));
    }
    // round cap at the end, then back along the other side
    let r_end = f(widths[n - 1] / 2.0).max(0.1);
    let e = side(n - 1, -1.0);
    let _ = write!(out, "A{r_end} {r_end} 0 0 0 {} {}", f(e.x), f(e.y));
    for i in (0..n - 1).rev() {
        let p = side(i, -1.0);
        let _ = write!(out, "L{} {}", f(p.x), f(p.y));
    }
    let r_start = f(widths[0] / 2.0).max(0.1);
    let _ = write!(out, "A{r_start} {r_start} 0 0 0 {} {}Z", f(a.x), f(a.y));
}

fn circle(p: P, r: f32, out: &mut String) {
    let _ = write!(
        out,
        "M{:.1} {:.1}A{r:.1} {r:.1} 0 1 0 {:.1} {:.1}A{r:.1} {r:.1} 0 1 0 {:.1} {:.1}Z",
        p.x - r,
        p.y,
        p.x + r,
        p.y,
        p.x - r,
        p.y
    );
}

/// The filled outline of a whole layer.
pub fn layer_outline(layer: &Layer, brush: &Brush, salt: u32) -> String {
    let mut out = String::new();
    for (k, stroke) in layer.strokes.iter().enumerate() {
        stroke_outline(stroke, layer.width, brush, salt.wrapping_add(k as u32 * 7919), &mut out);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::glyph::syllable_layers;
    use crate::phonology::{Syllable, Tone};

    fn outlines(b: &Brush) -> Vec<String> {
        let s = Syllable::from_pinyin("zhuang").unwrap().with_tone(Tone::First);
        syllable_layers(&s).iter().map(|l| layer_outline(l, b, 1)).collect()
    }

    #[test]
    fn every_preset_draws_clean_paths() {
        for b in [Brush::GEOMETRIC, Brush::PEN, Brush::BRUSH, Brush::hand(0.5)] {
            for d in outlines(&b) {
                assert!(!d.is_empty());
                assert!(!d.contains("NaN") && !d.contains("inf"), "{d}");
            }
        }
    }

    #[test]
    fn geometric_strokes_keep_constant_width() {
        // a straight 横 is a single rectangle with two round caps
        let stroke: Stroke = vec![Seg::M(10.0, 50.0), Seg::L(90.0, 50.0)];
        let mut d = String::new();
        stroke_outline(&stroke, 6.0, &Brush::GEOMETRIC, 0, &mut d);
        assert_eq!(d, "M10 53L90 53A3 3 0 0 0 90 47L10 47A3 3 0 0 0 10 53Z");
    }

    #[test]
    fn reads_strokes_like_a_calligrapher() {
        let read_of = |stroke: Stroke| {
            let s = sample(&stroke, &Brush::BRUSH);
            let hook = hook_corner(&s.pts, &s.corners);
            read(&s.pts, &s.corners, hook)
        };
        // 撇
        let r = read_of(vec![Seg::M(60.0, 20.0), Seg::Q(55.0, 60.0, 20.0, 80.0)]);
        assert_eq!(r.end, End::Sweep);
        // 竖钩 ends in a hook
        let r = read_of(vec![Seg::M(50.0, 20.0), Seg::L(50.0, 80.0), Seg::L(42.0, 74.0)]);
        assert!(r.hook_from.is_some());
        // a flattened 乙 still ends in a (shallow) hook
        let r = read_of(vec![
            Seg::M(49.6, 24.3),
            Seg::L(86.6, 24.3),
            Seg::L(51.5, 33.1),
            Seg::Q(45.8, 36.6, 60.0, 36.6),
            Seg::L(86.6, 36.6),
            Seg::L(91.3, 34.1),
        ]);
        assert!(r.hook_from.is_some());
        assert_eq!(r.end, End::Blunt);
        // 乙 (横折弯钩) ends in a hook, its bottom bar does not taper away
        let r = read_of(vec![
            Seg::M(20.0, 20.0),
            Seg::L(80.0, 20.0),
            Seg::L(25.0, 70.0),
            Seg::Q(18.0, 90.0, 40.0, 90.0),
            Seg::L(80.0, 90.0),
            Seg::L(88.0, 74.0),
        ]);
        assert!(r.hook_from.is_some());
        assert_eq!(r.end, End::Blunt);
        // 点
        let r = read_of(vec![Seg::M(40.0, 40.0), Seg::L(48.0, 50.0)]);
        assert_eq!(r.end, End::Dot);
        // 捺
        let r = read_of(vec![Seg::M(30.0, 30.0), Seg::Q(50.0, 60.0, 80.0, 80.0)]);
        assert_eq!(r.end, End::Foot);
        // 提
        let r = read_of(vec![Seg::M(30.0, 70.0), Seg::L(60.0, 50.0)]);
        assert_eq!(r.end, End::Rise);
    }
}
