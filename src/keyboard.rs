//! Logic for the on-screen keyboard: a syllable is typed as its components,
//! in block order (initial, medial, rhyme), and at every step only the keys
//! that can still lead to a real Mandarin syllable are offered.

use crate::parse::valid_syllables;
use crate::phonology::{Coda, Consonant, Syllable, Tone, Vowel};
use std::sync::OnceLock;

/// One key of the keyboard: one component of the script.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Key {
    Initial(Consonant),
    /// ㄧ ㄨ ㄩ: a medial, or the whole final when nothing follows
    Glide(Vowel),
    /// one of the thirteen rhymes (vowel, ending, r for ㄦ)
    Rhyme(Vowel, Option<Coda>, bool),
}

impl Key {
    pub fn zhuyin(self) -> &'static str {
        match self {
            Key::Initial(c) => c.zhuyin(),
            Key::Glide(v) => v.zhuyin(),
            Key::Rhyme(v, c, r) => Syllable { nucleus: Some(v), coda: c, rhotic: r, ..Syllable::empty() }.rhyme_zhuyin(),
        }
    }

    /// The key's sound in pinyin (b, zh, i, ü, ai, er …).
    pub fn pinyin(self) -> String {
        match self {
            Key::Initial(c) => c.pinyin().to_string(),
            Key::Glide(v) => v.pinyin().to_string(),
            Key::Rhyme(v, c, r) => Syllable { nucleus: Some(v), coda: c, rhotic: r, ..Syllable::empty() }.rhyme_name(),
        }
    }

    /// Position in zhuyin's own order (ㄅㄆㄇㄈ … ㄧㄨㄩ … ㄚㄛㄜㄝ …).
    fn order(self) -> usize {
        const ORDER: &str = "ㄅㄆㄇㄈㄉㄊㄋㄌㄍㄎㄏㄐㄑㄒㄓㄔㄕㄖㄗㄘㄙㄧㄨㄩㄚㄛㄜㄝㄞㄟㄠㄡㄢㄣㄤㄥㄦ";
        let z = self.zhuyin().chars().next().unwrap_or(' ');
        ORDER.chars().position(|c| c == z).unwrap_or(99)
    }
}

/// One place on the keyboard: a component key or a tone key.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Slot {
    Key(Key),
    Tone(Tone),
}

/// The key rows, adapted from the standard zhuyin keyboard (大千), whose
/// columns group the consonants by place (ㄅㄆㄇㄈ, ㄉㄊㄋㄌ, ㄍㄎㄏ, ㄐㄑㄒ,
/// ㄔㄕㄖ, ㄗㄘㄙ), then ㄧㄨㄩ and the rhymes. All five tones are gathered in
/// the middle of the top row. The bottom row leaves room for the ABC toggle
/// on its left and backspace on its right.
const BOARD: [&str; 4] = [
    "ㄅㄉㄓˉˊˇˋ˙ㄚㄞㄢ",
    "ㄆㄊㄍㄐㄔㄗㄧㄛㄟㄣㄦ",
    "ㄇㄋㄎㄑㄕㄘㄙㄨㄜㄠㄤ",
    "ㄈㄌㄏㄒㄖㄩㄝㄡㄥ",
];

/// The rhyme key for a zhuyin rhyme letter.
fn rhyme_key(c: char) -> Option<Key> {
    use Vowel::*;
    let (v, coda, r) = match c {
        'ㄚ' => (A, None, false),
        'ㄛ' => (O, None, false),
        'ㄜ' => (E, None, false),
        'ㄝ' => (Ehat, None, false),
        'ㄞ' => (A, Some(Coda::I), false),
        'ㄟ' => (Ehat, Some(Coda::I), false),
        'ㄠ' => (A, Some(Coda::U), false),
        'ㄡ' => (O, Some(Coda::U), false),
        'ㄢ' => (A, Some(Coda::N), false),
        'ㄣ' => (E, Some(Coda::N), false),
        'ㄤ' => (A, Some(Coda::Ng), false),
        'ㄥ' => (E, Some(Coda::Ng), false),
        'ㄦ' => (E, None, true),
        _ => return None,
    };
    Some(Key::Rhyme(v, coda, r))
}

fn slot(c: char) -> Option<Slot> {
    Some(match c {
        'ˉ' => Slot::Tone(Tone::First),
        'ˊ' => Slot::Tone(Tone::Second),
        'ˇ' => Slot::Tone(Tone::Third),
        'ˋ' => Slot::Tone(Tone::Fourth),
        '˙' => Slot::Tone(Tone::Neutral),
        'ㄧ' => Slot::Key(Key::Glide(Vowel::I)),
        'ㄨ' => Slot::Key(Key::Glide(Vowel::U)),
        'ㄩ' => Slot::Key(Key::Glide(Vowel::Yu)),
        _ => match Consonant::from_zhuyin(c) {
            Some(k) => Slot::Key(Key::Initial(k)),
            None => Slot::Key(rhyme_key(c)?),
        },
    })
}

/// The fixed key layout, row by row.
pub fn layout() -> Vec<Vec<Slot>> {
    BOARD.iter().map(|row| row.chars().filter_map(slot).collect()).collect()
}

/// The numbers and punctuation board (full-width Chinese punctuation).
pub const NUMBERS: [&str; 4] = [
    "1 2 3 4 5 6 7 8 9 0 .",
    "， 。 、 ？ ！ ： ； 「 」 『 』",
    "（ ） 《 》 … — ～ · % + =",
    "@ # ¥ $ & * / “ ”",
];

/// The Latin (ABC) board; ⇧ for capitals sits in front of the third row.
pub const LATIN: [&str; 4] = [
    "q w e r t y u i o p",
    "a s d f g h j k l",
    "z x c v b n m",
    ", . ? ! ' - : ( )",
];

/// The emoji board.
pub const EMOJI: [&str; 4] = [
    "😀 😂 🥰 😊 😎 🤔 😮 😢 😡 👍 👎",
    "❤️ 🎉 🙏 👏 🔥 ✨ 🌟 🌸 🍵 🍜 🥟",
    "🐼 🐉 🧧 🏮 🎋 🌙 ☀️ ⛰️ 🏠 🚀",
    "💡 📚 ✏️ 🎵 ⚽ 🎁 ✅ ❌ 👋",
];

/// The ㄦ key: the rhyme er, or 儿化 after a finished final.
pub const ER: Key = Key::Rhyme(Vowel::E, None, true);

/// Can 儿化 be added to what has been typed?
pub fn can_add_r(typed: &[Key]) -> bool {
    match typed.last() {
        Some(Key::Rhyme(_, _, r)) => !r,
        Some(Key::Glide(_)) => is_complete(typed),
        _ => false,
    }
}

/// The keys that spell a syllable, in typing order.
pub fn spell(s: &Syllable) -> Option<Vec<Key>> {
    let mut keys = Vec::new();
    if let Some(c) = s.initial {
        keys.push(Key::Initial(c));
    }
    if let Some(m) = s.medial {
        keys.push(Key::Glide(m));
    }
    match s.nucleus {
        Some(n @ (Vowel::I | Vowel::U | Vowel::Yu)) => keys.push(Key::Glide(n)),
        Some(n) => keys.push(Key::Rhyme(n, s.coda, s.rhotic && s.coda.is_none() && n == Vowel::E)),
        // syllabic nasals are not on the keyboard
        None if s.coda.is_some() => return None,
        None => {}
    }
    Some(keys)
}

fn spellings() -> &'static Vec<Vec<Key>> {
    static ALL: OnceLock<Vec<Vec<Key>>> = OnceLock::new();
    ALL.get_or_init(|| valid_syllables().iter().filter_map(spell).collect())
}

/// Keys that can follow what has been typed so far, in zhuyin order.
pub fn next_keys(typed: &[Key]) -> Vec<Key> {
    let mut out: Vec<Key> = Vec::new();
    for keys in spellings() {
        if keys.len() > typed.len() && keys[..typed.len()] == *typed {
            let k = keys[typed.len()];
            if !out.contains(&k) {
                out.push(k);
            }
        }
    }
    out.sort_by_key(|k| k.order());
    out
}

/// Is `typed` a whole syllable already (so a tone makes a real word)?
pub fn is_complete(typed: &[Key]) -> bool {
    spellings().iter().any(|k| k == typed)
}

/// Build the syllable from the keys typed so far, whatever they are.
pub fn compose(typed: &[Key], rhotic: bool, tone: Tone) -> Syllable {
    let mut s = Syllable::empty();
    for &k in typed {
        match k {
            Key::Initial(c) => s.initial = Some(c),
            Key::Glide(v) => s.medial = Some(v),
            Key::Rhyme(n, c, r) => {
                s.nucleus = Some(n);
                s.coda = c;
                s.rhotic = r;
            }
        }
    }
    if s.nucleus.is_none() {
        // a lone glide is the whole final: ㄧ yi, ㄐㄩ ju
        s.nucleus = s.medial.take();
    }
    s.rhotic |= rhotic;
    s.with_tone(tone)
}

/// The text to append to the input for a committed syllable: zhuyin with an
/// explicit tone mark, so consecutive syllables never run together.
pub fn commit_text(s: &Syllable) -> String {
    let mut z = s.zhuyin();
    if s.tone == Tone::First {
        z.push('ˉ');
    }
    z
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phonology::{Manner, Place};

    fn initial(p: &str) -> Key {
        Key::Initial(Consonant::from_pinyin(p).unwrap())
    }

    #[test]
    fn first_keys_include_initials_glides_and_rhymes() {
        let k = next_keys(&[]);
        assert!(k.contains(&initial("b")));
        assert!(k.contains(&Key::Glide(Vowel::Yu)));
        assert!(k.contains(&Key::Rhyme(Vowel::A, Some(Coda::N), false)));
        assert_eq!(k.first(), Some(&initial("b")));
    }

    #[test]
    fn only_plausible_keys_follow() {
        // after ㄐ only ㄧ and ㄩ can come
        let after_j = next_keys(&[initial("j")]);
        assert_eq!(after_j, vec![Key::Glide(Vowel::I), Key::Glide(Vowel::Yu)]);
        // after ㄅ there is no ㄩ
        let after_b = next_keys(&[initial("b")]);
        assert!(!after_b.contains(&Key::Glide(Vowel::Yu)));
        assert!(after_b.contains(&Key::Rhyme(Vowel::A, None, false)));
        // after ㄓㄨ: ㄚ ㄛ ㄞ ㄟ ㄢ ㄣ ㄤ ㄥ
        let zhu = next_keys(&[initial("zh"), Key::Glide(Vowel::U)]);
        let z: String = zhu.iter().map(|k| k.zhuyin()).collect();
        assert_eq!(z, "ㄚㄛㄞㄟㄢㄣㄤㄥ");
    }

    #[test]
    fn the_layout_has_every_component_once() {
        let rows = layout();
        let lens: Vec<usize> = rows.iter().map(|r| r.len()).collect();
        assert_eq!(lens, [11, 11, 11, 9]);
        // the five tones sit in the middle of the top row
        let tones: Vec<usize> =
            rows[0].iter().enumerate().filter(|(_, s)| matches!(s, Slot::Tone(_))).map(|(i, _)| i).collect();
        assert_eq!(tones, [3, 4, 5, 6, 7]);
        let slots: Vec<Slot> = rows.into_iter().flatten().collect();
        assert_eq!(slots.len(), 42);
        // the other boards fit the same rows (row 4 shares space with ABC and ⌫)
        for board in [NUMBERS, EMOJI, LATIN] {
            assert!(board[3].split_whitespace().count() <= 9);
        }
        let keys: Vec<Key> = slots
            .iter()
            .filter_map(|s| match s {
                Slot::Key(k) => Some(*k),
                Slot::Tone(_) => None,
            })
            .collect();
        assert_eq!(keys.len(), 37);
        // every key that can start or continue a syllable is on the board
        for k in next_keys(&[]) {
            assert!(keys.contains(&k), "{k:?}");
        }
        for (i, a) in keys.iter().enumerate() {
            assert!(!keys[i + 1..].contains(a), "duplicate {a:?}");
        }
        assert_eq!(ER.pinyin(), "er");
        assert_eq!(Key::Glide(Vowel::Yu).pinyin(), "ü");
    }

    #[test]
    fn composing_and_committing() {
        let typed = [initial("h"), Key::Rhyme(Vowel::A, Some(Coda::U), false)];
        assert!(is_complete(&typed));
        let s = compose(&typed, false, Tone::Third);
        assert_eq!(s.pinyin(), "hǎo");
        assert_eq!(commit_text(&s), "ㄏㄠˇ");
        // a lone glide becomes the final
        let s = compose(&[initial("j"), Key::Glide(Vowel::Yu)], false, Tone::First);
        assert_eq!(s.pinyin(), "jū");
        assert_eq!(commit_text(&s), "ㄐㄩˉ");
        // zhi: the initial alone is complete
        assert!(is_complete(&[Key::Initial(Consonant::new(Place::Retroflex, Manner::Plain))]));
        // committed text reads back as the same syllable
        let back = crate::parse::convert("ㄐㄩˉㄏㄠˇ");
        assert_eq!(back.len(), 2);
    }
}
