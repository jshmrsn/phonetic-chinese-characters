//! Chinese character data from the Unicode Unihan database: stroke counts,
//! and every reading of the standard characters with its frequency.
//!
//! `data/hanzi.txt` covers the 8,105 characters of the 通用规范汉字表
//! (kTGHZ2013) plus the frequency-ranked readings of kHanyuPinlu.
//! `data/strokes_cjk.txt` holds kTotalStrokes for every code point in
//! U+4E00–U+9FFF, one byte each (`'0' + strokes`).

use crate::parse::parse_pinyin_token;
use crate::phonology::Syllable;
use std::collections::HashMap;
use std::sync::OnceLock;

static HANZI: &str = include_str!("../data/hanzi.txt");
static STROKES: &str = include_str!("../data/strokes_cjk.txt");

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Match {
    pub ch: char,
    pub strokes: u8,
    /// occurrences of this character with this reading in the
    /// 现代汉语频率词典 corpus (kHanyuPinlu); 0 if it is not counted there
    pub freq: u32,
}

struct Index {
    by_sound: HashMap<(String, u8), Vec<Match>>,
    strokes: HashMap<char, u8>,
}

fn key(s: &Syllable) -> (String, u8) {
    let mut plain = *s;
    plain.rhotic = false;
    (plain.pinyin_plain(), s.tone.number())
}

fn index() -> &'static Index {
    static INDEX: OnceLock<Index> = OnceLock::new();
    INDEX.get_or_init(|| {
        let mut by_sound: HashMap<(String, u8), Vec<Match>> = HashMap::new();
        let mut strokes = HashMap::new();
        for line in HANZI.lines().filter(|l| !l.starts_with('#')) {
            let mut cols = line.split('\t');
            let (Some(ch), Some(n), Some(readings)) = (cols.next(), cols.next(), cols.next()) else {
                continue;
            };
            let Some(ch) = ch.chars().next() else { continue };
            let n: u8 = n.parse().unwrap_or(0);
            strokes.insert(ch, n);
            for r in readings.split(' ') {
                let (reading, freq) = match r.split_once(':') {
                    Some((p, f)) => (p, f.parse().unwrap_or(0)),
                    None => (r, 0),
                };
                let Some(syl) = parse_pinyin_token(reading).and_then(|v| v.into_iter().next()) else {
                    continue;
                };
                let list = by_sound.entry(key(&syl.0)).or_default();
                if !list.iter().any(|m| m.ch == ch) {
                    list.push(Match { ch, strokes: n, freq });
                }
            }
        }
        for list in by_sound.values_mut() {
            list.sort_by(|a, b| b.freq.cmp(&a.freq).then(a.strokes.cmp(&b.strokes)));
        }
        Index { by_sound, strokes }
    })
}

/// Total strokes of a character, if known.
pub fn strokes(c: char) -> Option<u8> {
    let cp = c as u32;
    if (0x4E00..0xA000).contains(&cp) {
        let b = STROKES.as_bytes()[(cp - 0x4E00) as usize];
        if b > b'0' {
            return Some(b - b'0');
        }
    }
    index().strokes.get(&c).copied()
}

/// Characters pronounced exactly like `s` (same syllable and tone), most
/// frequent first. 儿化 is ignored.
pub fn homophones(s: &Syllable) -> Vec<Match> {
    index().by_sound.get(&key(s)).cloned().unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phonology::Tone;

    fn syl(p: &str, t: Tone) -> Syllable {
        Syllable::from_pinyin(p).unwrap().with_tone(t)
    }

    #[test]
    fn stroke_counts() {
        assert_eq!(strokes('一'), Some(1));
        assert_eq!(strokes('好'), Some(6));
        assert_eq!(strokes('龍'), Some(16));
        assert_eq!(strokes('a'), None);
    }

    #[test]
    fn homophones_by_frequency() {
        let hao = homophones(&syl("hao", Tone::Third));
        assert_eq!(hao.first().map(|m| m.ch), Some('好'));
        let shi = homophones(&syl("shi", Tone::Fourth));
        assert!(shi.len() > 20);
        assert_eq!(shi.first().map(|m| m.ch), Some('是'));
        // 行 is found under both of its readings
        assert!(homophones(&syl("xing", Tone::Second)).iter().any(|m| m.ch == '行'));
        assert!(homophones(&syl("hang", Tone::Second)).iter().any(|m| m.ch == '行'));
        // neutral tone
        assert_eq!(homophones(&syl("de", Tone::Neutral)).first().map(|m| m.ch), Some('的'));
    }
}
