//! Mandarin syllable model.
//!
//! A syllable is `(initial)(medial) nucleus (coda)(r) + tone`, which is exactly
//! the set of slots a glyph has. Everything here is pure data: no
//! rendering, no input parsing.

// ---------------------------------------------------------------------------
// Consonants: place of articulation × manner of articulation
// ---------------------------------------------------------------------------

/// Where the closure happens. Each place is drawn as one enclosure frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Place {
    Labial,
    Alveolar,
    Dental,
    Retroflex,
    Palatal,
    Velar,
}

impl Place {
    pub const ALL: [Place; 6] = [
        Place::Labial,
        Place::Alveolar,
        Place::Dental,
        Place::Retroflex,
        Place::Palatal,
        Place::Velar,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Place::Labial => "Labial",
            Place::Alveolar => "Alveolar",
            Place::Dental => "Dental sibilant",
            Place::Retroflex => "Retroflex",
            Place::Palatal => "Palatal",
            Place::Velar => "Velar",
        }
    }

    /// The traditional component the frame is taken from.
    pub fn frame(self) -> &'static str {
        match self {
            Place::Labial => "匚",
            Place::Alveolar => "厂",
            Place::Dental => "凵",
            Place::Retroflex => "乚",
            Place::Palatal => "冂",
            Place::Velar => "𠃌",
        }
    }

    /// Why this frame (the mouth is pictured facing left).
    pub fn idea(self) -> &'static str {
        match self {
            Place::Labial => "closed at the front: the lips",
            Place::Alveolar => "closed top-front: tongue tip on the gum ridge",
            Place::Dental => "closed below: tongue tip down behind the lower teeth",
            Place::Retroflex => "one rising hook: tongue tip curled up and back",
            Place::Palatal => "a roof over everything: tongue body on the hard palate",
            Place::Velar => "one stroke closed top-back: tongue root on the soft palate",
        }
    }
}

/// How the air flows. Each manner is one inner mark placed inside the frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Manner {
    Plain,
    Aspirated,
    Nasal,
    Fricative,
    Liquid,
}

impl Manner {
    pub const ALL: [Manner; 5] = [
        Manner::Plain,
        Manner::Aspirated,
        Manner::Nasal,
        Manner::Fricative,
        Manner::Liquid,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Manner::Plain => "Unaspirated",
            Manner::Aspirated => "Aspirated",
            Manner::Nasal => "Nasal",
            Manner::Fricative => "Fricative",
            Manner::Liquid => "Liquid",
        }
    }

    pub fn mark(self) -> &'static str {
        match self {
            Manner::Plain => "(empty)",
            Manner::Aspirated => "一",
            Manner::Nasal => "丶",
            Manner::Fricative => "丿",
            Manner::Liquid => "丨",
        }
    }

    pub fn idea(self) -> &'static str {
        match self {
            Manner::Plain => "nothing inside: a clean stop",
            Manner::Aspirated => "one level stroke: a puff of breath",
            Manner::Nasal => "a single dot: the voice resonating in the nose",
            Manner::Fricative => "a slash: air squeezed through a narrow gap",
            Manner::Liquid => "one upright stroke: the tongue held in place, air flows round it",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Consonant {
    pub place: Place,
    pub manner: Manner,
}

/// (pinyin, zhuyin, place, manner)
const CONSONANTS: &[(&str, &str, Place, Manner)] = &[
    ("b", "ㄅ", Place::Labial, Manner::Plain),
    ("p", "ㄆ", Place::Labial, Manner::Aspirated),
    ("m", "ㄇ", Place::Labial, Manner::Nasal),
    ("f", "ㄈ", Place::Labial, Manner::Fricative),
    ("d", "ㄉ", Place::Alveolar, Manner::Plain),
    ("t", "ㄊ", Place::Alveolar, Manner::Aspirated),
    ("n", "ㄋ", Place::Alveolar, Manner::Nasal),
    ("l", "ㄌ", Place::Alveolar, Manner::Liquid),
    ("z", "ㄗ", Place::Dental, Manner::Plain),
    ("c", "ㄘ", Place::Dental, Manner::Aspirated),
    ("s", "ㄙ", Place::Dental, Manner::Fricative),
    ("zh", "ㄓ", Place::Retroflex, Manner::Plain),
    ("ch", "ㄔ", Place::Retroflex, Manner::Aspirated),
    ("sh", "ㄕ", Place::Retroflex, Manner::Fricative),
    ("r", "ㄖ", Place::Retroflex, Manner::Liquid),
    ("j", "ㄐ", Place::Palatal, Manner::Plain),
    ("q", "ㄑ", Place::Palatal, Manner::Aspirated),
    ("x", "ㄒ", Place::Palatal, Manner::Fricative),
    ("g", "ㄍ", Place::Velar, Manner::Plain),
    ("k", "ㄎ", Place::Velar, Manner::Aspirated),
    ("h", "ㄏ", Place::Velar, Manner::Fricative),
    // Only ever a coda (or a syllabic interjection), never an initial.
    ("ng", "ㄫ", Place::Velar, Manner::Nasal),
];

impl Consonant {
    pub const fn new(place: Place, manner: Manner) -> Self {
        Consonant { place, manner }
    }

    /// Whether this place/manner combination is a Mandarin sound at all.
    pub fn exists(self) -> bool {
        self.entry().is_some()
    }

    pub fn from_pinyin(s: &str) -> Option<Self> {
        CONSONANTS
            .iter()
            .find(|e| e.0 == s)
            .map(|&(_, _, place, manner)| Consonant { place, manner })
    }

    pub fn from_zhuyin(c: char) -> Option<Self> {
        CONSONANTS
            .iter()
            .find(|e| e.1.starts_with(c))
            .map(|&(_, _, place, manner)| Consonant { place, manner })
    }

    fn entry(self) -> Option<&'static (&'static str, &'static str, Place, Manner)> {
        CONSONANTS
            .iter()
            .find(|e| e.2 == self.place && e.3 == self.manner)
    }

    pub fn pinyin(self) -> &'static str {
        self.entry().map(|e| e.0).unwrap_or("?")
    }

    pub fn zhuyin(self) -> &'static str {
        self.entry().map(|e| e.1).unwrap_or("?")
    }

    /// z c s zh ch sh r: the initials that take the "empty" -i rhyme.
    pub fn takes_apical_vowel(self) -> bool {
        matches!(self.place, Place::Dental | Place::Retroflex)
    }

    pub const N: Consonant = Consonant::new(Place::Alveolar, Manner::Nasal);
    pub const NG: Consonant = Consonant::new(Place::Velar, Manner::Nasal);
    pub const M: Consonant = Consonant::new(Place::Labial, Manner::Nasal);
    pub const H: Consonant = Consonant::new(Place::Velar, Manner::Fricative);
}

// ---------------------------------------------------------------------------
// Vowels, codas, tones
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Vowel {
    A,
    O,
    E,
    /// ê, the open e of 也 / 月 (pinyin writes it "e" after i and ü)
    Ehat,
    I,
    U,
    /// ü
    Yu,
}

impl Vowel {
    pub fn pinyin(self) -> &'static str {
        match self {
            Vowel::A => "a",
            Vowel::O => "o",
            Vowel::E => "e",
            Vowel::Ehat => "ê",
            Vowel::I => "i",
            Vowel::U => "u",
            Vowel::Yu => "ü",
        }
    }

    pub fn zhuyin(self) -> &'static str {
        match self {
            Vowel::A => "ㄚ",
            Vowel::O => "ㄛ",
            Vowel::E => "ㄜ",
            Vowel::Ehat => "ㄝ",
            Vowel::I => "ㄧ",
            Vowel::U => "ㄨ",
            Vowel::Yu => "ㄩ",
        }
    }

    pub fn component(self) -> &'static str {
        match self {
            Vowel::A => "八",
            Vowel::O => "又",
            Vowel::E => "了",
            Vowel::Ehat => "人",
            Vowel::I => "乙",
            Vowel::U => "乂",
            Vowel::Yu => "于",
        }
    }

    pub fn idea(self) -> &'static str {
        match self {
            Vowel::A => "八 bā: the jaw wide open",
            Vowel::O => "又: a rounded loop",
            Vowel::E => "了 le: the unrounded mid vowel of 哥, 根 and 冷",
            Vowel::Ehat => "the front mid vowel of 也 yě, 谢 xiè and 美 měi: half-closed 八",
            Vowel::I => "乙 yǐ: one stroke",
            Vowel::U => "zhuyin's ㄨ: crossed strokes for pursed lips",
            Vowel::Yu => "于 yú: a shape of its own, as zhuyin gives ü its own ㄩ",
        }
    }
}

/// What closes the syllable.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Coda {
    /// off-glide i (ai, ei, uai, uei)
    I,
    /// off-glide u (ao, ou, iao, iou)
    U,
    N,
    Ng,
    /// only in interjections (呣 m, 噷 hm)
    M,
}

impl Coda {
    pub fn pinyin(self) -> &'static str {
        match self {
            Coda::I => "i",
            Coda::U => "u",
            Coda::N => "n",
            Coda::Ng => "ng",
            Coda::M => "m",
        }
    }

    /// Nasal codas are written with the same component as the consonant.
    pub fn consonant(self) -> Option<Consonant> {
        match self {
            Coda::N => Some(Consonant::N),
            Coda::Ng => Some(Consonant::NG),
            Coda::M => Some(Consonant::M),
            _ => None,
        }
    }

    /// The single stroke a rhyme adds to its vowel for this ending.
    pub fn stroke(self) -> &'static str {
        match self {
            Coda::I => "提 ㇀",
            Coda::U => "卧钩 ㇃",
            Coda::N => "横 一",
            Coda::Ng => "横折钩 𠃌",
            Coda::M => "匚 + 丶",
        }
    }

    pub fn idea(self) -> &'static str {
        match self {
            Coda::I => "a rising stroke: the tongue rises toward i",
            Coda::U => "a bowl: the lips round toward u",
            Coda::N => "a level bar: the tongue tip seals the mouth",
            Coda::Ng => "the same bar closed at the back, like the velar frame",
            Coda::M => "only in interjections (呣, 噷)",
        }
    }
}

/// The 儿化 / er ending stroke.
pub const R_STROKE: &str = "竖弯钩 乚";
pub const R_IDEA: &str = "the retroflex curl: the tongue tip curls up";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Tone {
    First,
    Second,
    Third,
    Fourth,
    Neutral,
}

impl Tone {
    pub const ALL: [Tone; 5] = [
        Tone::First,
        Tone::Second,
        Tone::Third,
        Tone::Fourth,
        Tone::Neutral,
    ];

    pub fn from_number(n: u8) -> Tone {
        match n {
            1 => Tone::First,
            2 => Tone::Second,
            3 => Tone::Third,
            4 => Tone::Fourth,
            _ => Tone::Neutral,
        }
    }

    pub fn number(self) -> u8 {
        match self {
            Tone::First => 1,
            Tone::Second => 2,
            Tone::Third => 3,
            Tone::Fourth => 4,
            Tone::Neutral => 5,
        }
    }

    pub fn pinyin_mark(self) -> &'static str {
        match self {
            Tone::First => "¯",
            Tone::Second => "´",
            Tone::Third => "ˇ",
            Tone::Fourth => "`",
            Tone::Neutral => "(none)",
        }
    }

    pub fn stroke_name(self) -> &'static str {
        match self {
            Tone::First => "横 level stroke",
            Tone::Second => "提 rising stroke",
            Tone::Third => "点 + 提 dip",
            Tone::Fourth => "点 falling stroke",
            Tone::Neutral => "no mark",
        }
    }
}

// ---------------------------------------------------------------------------
// Syllable
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Syllable {
    pub initial: Option<Consonant>,
    /// i, u or ü gliding into the nucleus
    pub medial: Option<Vowel>,
    /// `None` for the "empty" rhyme of zi/ci/si/zhi/chi/shi/ri and for
    /// syllabic nasals (m, n, ng, hm, hng)
    pub nucleus: Option<Vowel>,
    pub coda: Option<Coda>,
    /// 儿化 / er
    pub rhotic: bool,
    pub tone: Tone,
}

/// Every Pinyin final with its zhuyin-style decomposition into
/// medial (ㄧ ㄨ ㄩ) + rhyme, the rhyme being vowel + ending.
/// in = ㄧㄣ, ing = ㄧㄥ, ün = ㄩㄣ, ong = ㄨㄥ, iong = ㄩㄥ, iu = ㄧㄡ, ui = ㄨㄟ.
pub const FINALS: &[(&str, Option<Vowel>, Vowel, Option<Coda>)] = {
    use Coda as C;
    use Vowel::*;
    &[
        ("a", None, A, None),
        ("o", None, O, None),
        ("e", None, E, None),
        ("ê", None, Ehat, None),
        ("ai", None, A, Some(C::I)),
        // ei [eɪ] starts on the front vowel of ie / ê, not the e of 哥
        ("ei", None, Ehat, Some(C::I)),
        ("ao", None, A, Some(C::U)),
        ("ou", None, O, Some(C::U)),
        ("an", None, A, Some(C::N)),
        ("en", None, E, Some(C::N)),
        ("ang", None, A, Some(C::Ng)),
        ("eng", None, E, Some(C::Ng)),
        // zhuyin ㄨㄥ: the same final as weng, written ong after a consonant
        ("ong", Some(U), E, Some(C::Ng)),
        ("i", None, I, None),
        ("ia", Some(I), A, None),
        ("io", Some(I), O, None),
        ("ie", Some(I), Ehat, None),
        ("iai", Some(I), A, Some(C::I)),
        ("iao", Some(I), A, Some(C::U)),
        ("iou", Some(I), O, Some(C::U)),
        ("ian", Some(I), A, Some(C::N)),
        ("in", Some(I), E, Some(C::N)),
        ("iang", Some(I), A, Some(C::Ng)),
        ("ing", Some(I), E, Some(C::Ng)),
        ("iong", Some(Yu), E, Some(C::Ng)),
        ("u", None, U, None),
        ("ua", Some(U), A, None),
        ("uo", Some(U), O, None),
        ("uai", Some(U), A, Some(C::I)),
        ("uei", Some(U), Ehat, Some(C::I)),
        ("uan", Some(U), A, Some(C::N)),
        ("uen", Some(U), E, Some(C::N)),
        ("uang", Some(U), A, Some(C::Ng)),
        ("ueng", Some(U), E, Some(C::Ng)),
        ("ü", None, Yu, None),
        ("üe", Some(Yu), Ehat, None),
        ("üan", Some(Yu), A, Some(C::N)),
        ("ün", Some(Yu), E, Some(C::N)),
    ]
};

impl Syllable {
    pub fn empty() -> Self {
        Syllable {
            initial: None,
            medial: None,
            nucleus: None,
            coda: None,
            rhotic: false,
            tone: Tone::Neutral,
        }
    }

    pub fn with_tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }

    /// Analyse a toneless, lower-case pinyin syllable (`ü` written as `ü`).
    /// Undoes all of pinyin's spelling shortcuts: y/w, iu, ui, un, ju→jü …
    pub fn from_pinyin(s: &str) -> Option<Syllable> {
        let mut syl = Syllable::empty();
        match s {
            "m" => {
                syl.coda = Some(Coda::M);
                return Some(syl);
            }
            "n" => {
                syl.coda = Some(Coda::N);
                return Some(syl);
            }
            "ng" => {
                syl.coda = Some(Coda::Ng);
                return Some(syl);
            }
            "hm" => {
                syl.initial = Some(Consonant::H);
                syl.coda = Some(Coda::M);
                return Some(syl);
            }
            "hng" => {
                syl.initial = Some(Consonant::H);
                syl.coda = Some(Coda::Ng);
                return Some(syl);
            }
            "er" => {
                syl.nucleus = Some(Vowel::E);
                syl.rhotic = true;
                return Some(syl);
            }
            _ => {}
        }
        if let Some(stem) = s.strip_suffix('r') {
            if let Some(mut base) = Self::from_pinyin_plain(stem) {
                base.rhotic = true;
                return Some(base);
            }
        }
        Self::from_pinyin_plain(s)
    }

    fn from_pinyin_plain(s: &str) -> Option<Syllable> {
        let mut syl = Syllable::empty();
        let fin: String;
        if let Some(rest) = s.strip_prefix('y') {
            fin = if rest.starts_with('i') {
                rest.to_string()
            } else if let Some(r) = rest.strip_prefix('u') {
                format!("ü{r}")
            } else {
                format!("i{rest}")
            };
        } else if let Some(rest) = s.strip_prefix('w') {
            fin = if rest == "u" {
                "u".to_string()
            } else {
                format!("u{rest}")
            };
        } else {
            let (initial, rest) = split_initial(s);
            syl.initial = initial;
            let mut f = rest.to_string();
            if let Some(c) = initial {
                if c.place == Place::Palatal {
                    if let Some(r) = f.strip_prefix('u') {
                        f = format!("ü{r}");
                    }
                }
                // "lue"/"nue" are common keyboard spellings of lüe/nüe
                if f == "ue" {
                    f = "üe".to_string();
                }
                if f == "i" && c.takes_apical_vowel() {
                    return Some(syl);
                }
            }
            f = match f.as_str() {
                "iu" => "iou".to_string(),
                "ui" => "uei".to_string(),
                "un" => "uen".to_string(),
                _ => f,
            };
            fin = f;
        }
        let (_, medial, nucleus, coda) = FINALS.iter().find(|e| e.0 == fin)?;
        syl.medial = *medial;
        syl.nucleus = Some(*nucleus);
        syl.coda = *coda;
        Some(syl)
    }

    /// The final fully spelled out: "iou", "uei", "üan", "ê" …
    pub fn final_key(&self) -> String {
        let mut s = String::new();
        if let Some(m) = self.medial {
            s.push_str(m.pinyin());
        }
        match self.nucleus {
            Some(Vowel::Ehat) if self.medial.is_some() || self.coda.is_some() => s.push('e'),
            Some(v) => s.push_str(v.pinyin()),
            None => {}
        }
        match self.coda {
            Some(Coda::U) if self.nucleus == Some(Vowel::A) => s.push('o'),
            Some(c) => s.push_str(c.pinyin()),
            None => {}
        }
        // pinyin's own spellings of the zhuyin finals ㄧㄣ ㄧㄥ ㄩㄣ ㄩㄥ ㄨㄥ
        match s.as_str() {
            "ien" => "in".to_string(),
            "ieng" => "ing".to_string(),
            "üen" => "ün".to_string(),
            "üeng" => "iong".to_string(),
            "ueng" if self.initial.is_some() => "ong".to_string(),
            _ => s,
        }
    }

    /// The rhyme in zhuyin: one of ㄚㄛㄜㄝㄞㄟㄠㄡㄢㄣㄤㄥㄦ, or ㄧㄨㄩ when
    /// the glide stands alone.
    pub fn rhyme_zhuyin(&self) -> &'static str {
        use Vowel::*;
        match (self.nucleus, self.coda) {
            (Some(E), None) if self.rhotic => "ㄦ",
            (Some(A), None) => "ㄚ",
            (Some(O), None) => "ㄛ",
            (Some(E), None) => "ㄜ",
            (Some(Ehat), None) => "ㄝ",
            (Some(A), Some(Coda::I)) => "ㄞ",
            (Some(Ehat), Some(Coda::I)) => "ㄟ",
            (Some(A), Some(Coda::U)) => "ㄠ",
            (Some(O), Some(Coda::U)) => "ㄡ",
            (Some(A), Some(Coda::N)) => "ㄢ",
            (Some(E), Some(Coda::N)) => "ㄣ",
            (Some(A), Some(Coda::Ng)) => "ㄤ",
            (Some(E), Some(Coda::Ng)) => "ㄥ",
            (Some(I), None) => "ㄧ",
            (Some(U), None) => "ㄨ",
            (Some(Yu), None) => "ㄩ",
            (None, Some(Coda::M)) => "ㄇ",
            (None, Some(Coda::N)) => "ㄣ",
            (None, Some(Coda::Ng)) => "ㄫ",
            _ => "",
        }
    }

    /// The rhyme (vowel + ending) as one sound, e.g. "ao", "ong", "er".
    pub fn rhyme_name(&self) -> String {
        let Some(n) = self.nucleus else {
            return String::new();
        };
        let bare = Syllable {
            initial: None,
            medial: None,
            nucleus: Some(n),
            coda: self.coda,
            rhotic: false,
            tone: Tone::Neutral,
        };
        let mut s = bare.final_key();
        if self.rhotic {
            s.push('r');
        }
        s
    }

    /// Standard pinyin without tone mark.
    pub fn pinyin_plain(&self) -> String {
        let ini = self.initial.map(|c| c.pinyin()).unwrap_or("");
        let mut out = if self.nucleus.is_none() {
            match self.coda {
                Some(c) => format!("{ini}{}", c.pinyin()),
                None if self.initial.is_some() => format!("{ini}i"),
                None => String::new(),
            }
        } else {
            let fin = self.final_key();
            if self.initial.is_none() {
                spell_zero_initial(&fin)
            } else {
                let mut f = fin;
                if self.initial.map(|c| c.place) == Some(Place::Palatal) {
                    f = f.replace('ü', "u");
                }
                let f = match f.as_str() {
                    "iou" => "iu".to_string(),
                    "uei" => "ui".to_string(),
                    "uen" => "un".to_string(),
                    _ => f,
                };
                format!("{ini}{f}")
            }
        };
        if self.rhotic {
            out.push('r');
        }
        out
    }

    /// Standard pinyin with a tone mark.
    pub fn pinyin(&self) -> String {
        add_tone_mark(&self.pinyin_plain(), self.tone)
    }

    /// Standard Zhuyin (Bopomofo).
    pub fn zhuyin(&self) -> String {
        let mut s = String::new();
        if self.tone == Tone::Neutral {
            s.push('˙');
        }
        if let Some(c) = self.initial {
            s.push_str(c.zhuyin());
        }
        if let Some(m) = self.medial {
            s.push_str(m.zhuyin());
        }
        let rhyme = self.rhyme_zhuyin();
        s.push_str(rhyme);
        if self.rhotic && !s.ends_with('ㄦ') {
            s.push('ㄦ');
        }
        match self.tone {
            Tone::Second => s.push('ˊ'),
            Tone::Third => s.push('ˇ'),
            Tone::Fourth => s.push('ˋ'),
            _ => {}
        }
        s
    }
}

fn split_initial(s: &str) -> (Option<Consonant>, &str) {
    for p in ["zh", "ch", "sh"] {
        if let Some(rest) = s.strip_prefix(p) {
            return (Consonant::from_pinyin(p), rest);
        }
    }
    if let Some(first) = s.chars().next() {
        let mut buf = [0u8; 4];
        if let Some(c) = Consonant::from_pinyin(first.encode_utf8(&mut buf)) {
            return (Some(c), &s[first.len_utf8()..]);
        }
    }
    (None, s)
}

fn spell_zero_initial(fin: &str) -> String {
    if let Some(r) = fin.strip_prefix('ü') {
        return format!("yu{r}");
    }
    if let Some(r) = fin.strip_prefix('i') {
        return if r.is_empty() || r == "n" || r == "ng" {
            format!("y{fin}")
        } else {
            format!("y{r}")
        };
    }
    if let Some(r) = fin.strip_prefix('u') {
        return if r.is_empty() {
            "wu".to_string()
        } else {
            format!("w{r}")
        };
    }
    fin.to_string()
}

fn toned(c: char, tone: Tone) -> String {
    let n = tone.number();
    let pick = |v: [char; 4]| v[(n - 1) as usize].to_string();
    match c {
        'a' => pick(['ā', 'á', 'ǎ', 'à']),
        'e' => pick(['ē', 'é', 'ě', 'è']),
        'i' => pick(['ī', 'í', 'ǐ', 'ì']),
        'o' => pick(['ō', 'ó', 'ǒ', 'ò']),
        'u' => pick(['ū', 'ú', 'ǔ', 'ù']),
        'ü' => pick(['ǖ', 'ǘ', 'ǚ', 'ǜ']),
        'ê' if n == 2 => "ế".to_string(),
        'ê' if n == 4 => "ề".to_string(),
        'n' if n == 2 => "ń".to_string(),
        'n' if n == 3 => "ň".to_string(),
        'n' if n == 4 => "ǹ".to_string(),
        'm' if n == 2 => "ḿ".to_string(),
        _ => {
            let comb = ['\u{0304}', '\u{0301}', '\u{030C}', '\u{0300}'][(n - 1) as usize];
            format!("{c}{comb}")
        }
    }
}

/// Place a tone diacritic using the standard pinyin rules.
pub fn add_tone_mark(plain: &str, tone: Tone) -> String {
    if tone == Tone::Neutral || plain.is_empty() {
        return plain.to_string();
    }
    let chars: Vec<char> = plain.chars().collect();
    let is_vowel = |c: char| "aeiouüê".contains(c);
    let idx = chars
        .iter()
        .position(|&c| c == 'a')
        .or_else(|| chars.iter().position(|&c| c == 'e' || c == 'ê'))
        .or_else(|| {
            chars
                .windows(2)
                .position(|w| w[0] == 'o' && w[1] == 'u')
        })
        .or_else(|| chars.iter().rposition(|&c| is_vowel(c)))
        .or_else(|| chars.iter().position(|&c| c == 'm' || c == 'n'))
        .unwrap_or(0);
    chars
        .iter()
        .enumerate()
        .map(|(i, &c)| if i == idx { toned(c, tone) } else { c.to_string() })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(s: &str) -> Syllable {
        Syllable::from_pinyin(s).unwrap_or_else(|| panic!("failed to parse {s}"))
    }

    #[test]
    fn finals_table_round_trips() {
        for (key, medial, nucleus, coda) in FINALS {
            let syl = Syllable {
                // ong only exists after a consonant
                initial: (*key == "ong").then_some(Consonant::H),
                medial: *medial,
                nucleus: Some(*nucleus),
                coda: *coda,
                ..Syllable::empty()
            };
            assert_eq!(&syl.final_key(), key);
        }
    }

    #[test]
    fn undoes_pinyin_abbreviations() {
        assert_eq!(p("liu").final_key(), "iou");
        assert_eq!(p("gui").final_key(), "uei");
        assert_eq!(p("dun").final_key(), "uen");
        assert_eq!(p("ju").final_key(), "ü");
        assert_eq!(p("xuan").final_key(), "üan");
        assert_eq!(p("yu").final_key(), "ü");
        assert_eq!(p("you").final_key(), "iou");
        assert_eq!(p("wei").final_key(), "uei");
        assert_eq!(p("ye").final_key(), "ie");
        assert_eq!(p("lüe").final_key(), "üe");
        assert_eq!(p("lue").final_key(), "üe");
        assert_eq!(p("lu").final_key(), "u");
        assert!(p("zhi").nucleus.is_none());
        assert!(p("si").nucleus.is_none());
        assert_eq!(p("ji").nucleus, Some(Vowel::I));
    }

    #[test]
    fn respells_pinyin() {
        for s in [
            "a", "ba", "zhi", "ri", "yi", "wu", "yu", "ju", "lü", "nüe", "xue", "you", "liu",
            "wei", "gui", "wen", "dun", "yun", "jun", "yong", "xiong", "weng", "dong", "er",
            "huar", "yuan", "ying", "yin", "ng", "hm", "ê", "yo",
        ] {
            assert_eq!(p(s).pinyin_plain(), s, "round trip of {s}");
        }
    }

    #[test]
    fn tone_marks() {
        assert_eq!(p("hao").with_tone(Tone::Third).pinyin(), "hǎo");
        assert_eq!(p("gui").with_tone(Tone::Fourth).pinyin(), "guì");
        assert_eq!(p("liu").with_tone(Tone::Second).pinyin(), "liú");
        assert_eq!(p("lü").with_tone(Tone::Fourth).pinyin(), "lǜ");
        assert_eq!(p("dou").with_tone(Tone::First).pinyin(), "dōu");
        assert_eq!(p("n").with_tone(Tone::Second).pinyin(), "ń");
    }

    #[test]
    fn zhuyin_spelling() {
        assert_eq!(p("zhong").with_tone(Tone::First).zhuyin(), "ㄓㄨㄥ");
        assert_eq!(p("xiong").with_tone(Tone::Second).zhuyin(), "ㄒㄩㄥˊ");
        assert_eq!(p("xue").with_tone(Tone::Second).zhuyin(), "ㄒㄩㄝˊ");
        assert_eq!(p("shi").with_tone(Tone::Fourth).zhuyin(), "ㄕˋ");
        assert_eq!(p("de").zhuyin(), "˙ㄉㄜ");
        assert_eq!(p("er").with_tone(Tone::Fourth).zhuyin(), "ㄦˋ");
        assert_eq!(p("ying").with_tone(Tone::First).zhuyin(), "ㄧㄥ");
        assert_eq!(p("gui").with_tone(Tone::Fourth).zhuyin(), "ㄍㄨㄟˋ");
        assert_eq!(p("jin").with_tone(Tone::First).zhuyin(), "ㄐㄧㄣ");
        assert_eq!(p("yun").with_tone(Tone::Second).zhuyin(), "ㄩㄣˊ");
        assert_eq!(p("weng").with_tone(Tone::First).zhuyin(), "ㄨㄥ");
        assert_eq!(p("mei").with_tone(Tone::Third).zhuyin(), "ㄇㄟˇ");
        assert_eq!(p("ye").with_tone(Tone::Third).zhuyin(), "ㄧㄝˇ");
    }

    #[test]
    fn rhymes_follow_zhuyin() {
        // the same rhyme symbol, different medials
        assert_eq!(p("jin").rhyme_zhuyin(), "ㄣ");
        assert_eq!(p("dun").rhyme_zhuyin(), "ㄣ");
        assert_eq!(p("men").rhyme_zhuyin(), "ㄣ");
        assert_eq!(p("dong").rhyme_zhuyin(), "ㄥ");
        assert_eq!(p("xiong").rhyme_zhuyin(), "ㄥ");
        // e, ei and ê are three different sounds
        assert_eq!(p("me").nucleus, Some(Vowel::E));
        assert_eq!(p("mei").nucleus, Some(Vowel::Ehat));
        assert_eq!(p("ye").nucleus, Some(Vowel::Ehat));
        assert_ne!(p("me").rhyme_zhuyin(), p("mei").rhyme_zhuyin());
    }
}
