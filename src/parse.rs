//! Turning free text (Hanzi, Pinyin, Zhuyin, or a mix) into syllables.

use crate::phonology::{Consonant, Syllable, Tone};
use std::collections::HashSet;
use std::sync::OnceLock;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Origin {
    Hanzi,
    Pinyin,
    Zhuyin,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SylItem {
    pub syl: Syllable,
    /// The text this syllable was read from.
    pub source: String,
    pub origin: Origin,
    /// Set when tone sandhi changed the tone; holds the citation tone.
    pub sandhi_from: Option<Tone>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Item {
    Syl(SylItem),
    /// Punctuation or anything that is not Mandarin.
    Text(String),
    Space,
    Newline,
}

// ---------------------------------------------------------------------------
// Character classes
// ---------------------------------------------------------------------------

pub fn is_hanzi(c: char) -> bool {
    matches!(c as u32,
        0x3400..=0x4DBF | 0x4E00..=0x9FFF | 0xF900..=0xFAFF | 0x20000..=0x2EBEF | 0x30000..=0x3134F)
}

fn is_zhuyin(c: char) -> bool {
    matches!(c as u32, 0x3105..=0x312F | 0x31A0..=0x31BF) || is_zhuyin_tone(c)
}

fn is_zhuyin_tone(c: char) -> bool {
    matches!(c, 'ˉ' | 'ˊ' | 'ˇ' | 'ˋ' | '˙')
}

fn is_apostrophe(c: char) -> bool {
    matches!(c, '\'' | '’' | '‘' | 'ʼ')
}

fn is_combining(c: char) -> bool {
    matches!(c as u32, 0x0300..=0x036F)
}

fn is_pinyin_char(c: char) -> bool {
    c.is_ascii_alphabetic() || toned_vowel(c).is_some() || matches!(c, 'ü' | 'Ü' | 'ê' | 'Ê')
}

/// Decompose a precomposed pinyin letter into (base, tone number).
fn toned_vowel(c: char) -> Option<(char, u8)> {
    const TABLE: &[(char, char, u8)] = &[
        ('ā', 'a', 1), ('á', 'a', 2), ('ǎ', 'a', 3), ('à', 'a', 4),
        ('ē', 'e', 1), ('é', 'e', 2), ('ě', 'e', 3), ('è', 'e', 4),
        ('ī', 'i', 1), ('í', 'i', 2), ('ǐ', 'i', 3), ('ì', 'i', 4),
        ('ō', 'o', 1), ('ó', 'o', 2), ('ǒ', 'o', 3), ('ò', 'o', 4),
        ('ū', 'u', 1), ('ú', 'u', 2), ('ǔ', 'u', 3), ('ù', 'u', 4),
        ('ǖ', 'ü', 1), ('ǘ', 'ü', 2), ('ǚ', 'ü', 3), ('ǜ', 'ü', 4),
        ('ế', 'ê', 2), ('ề', 'ê', 4),
        ('ḿ', 'm', 2), ('ń', 'n', 2), ('ň', 'n', 3), ('ǹ', 'n', 4),
        ('Ā', 'a', 1), ('Á', 'a', 2), ('Ǎ', 'a', 3), ('À', 'a', 4),
        ('Ē', 'e', 1), ('É', 'e', 2), ('Ě', 'e', 3), ('È', 'e', 4),
        ('Ī', 'i', 1), ('Í', 'i', 2), ('Ǐ', 'i', 3), ('Ì', 'i', 4),
        ('Ō', 'o', 1), ('Ó', 'o', 2), ('Ǒ', 'o', 3), ('Ò', 'o', 4),
        ('Ū', 'u', 1), ('Ú', 'u', 2), ('Ǔ', 'u', 3), ('Ù', 'u', 4),
        ('Ǖ', 'ü', 1), ('Ǘ', 'ü', 2), ('Ǚ', 'ü', 3), ('Ǜ', 'ü', 4),
    ];
    TABLE.iter().find(|e| e.0 == c).map(|e| (e.1, e.2))
}

// ---------------------------------------------------------------------------
// Top level
// ---------------------------------------------------------------------------

pub fn convert(input: &str) -> Vec<Item> {
    let chars: Vec<char> = input.chars().collect();
    let mut items = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '\n' {
            items.push(Item::Newline);
            i += 1;
        } else if c.is_whitespace() {
            if !matches!(items.last(), Some(Item::Space) | Some(Item::Newline) | None) {
                items.push(Item::Space);
            }
            i += 1;
        } else if is_hanzi(c) {
            let start = i;
            while i < chars.len() && is_hanzi(chars[i]) {
                i += 1;
            }
            hanzi_run(&chars[start..i], &mut items);
        } else if is_zhuyin(c) {
            let start = i;
            while i < chars.len() && is_zhuyin(chars[i]) {
                i += 1;
            }
            zhuyin_run(&chars[start..i], &mut items);
        } else if is_pinyin_char(c) {
            let start = i;
            while i < chars.len() {
                let d = chars[i];
                let ok = is_pinyin_char(d)
                    || is_combining(d)
                    || d.is_ascii_digit()
                    || (is_apostrophe(d) && i + 1 < chars.len() && is_pinyin_char(chars[i + 1]))
                    || (d == ':' && i > start && matches!(chars[i - 1], 'u' | 'U'));
                if !ok {
                    break;
                }
                i += 1;
            }
            let token: String = chars[start..i].iter().collect();
            match parse_pinyin_token(&token) {
                Some(syls) => items.extend(syls.into_iter().map(|(syl, source)| {
                    Item::Syl(SylItem { syl, source, origin: Origin::Pinyin, sandhi_from: None })
                })),
                None => items.push(Item::Text(token)),
            }
        } else {
            let start = i;
            i += 1;
            while i < chars.len() && is_combining(chars[i]) {
                i += 1;
            }
            let t: String = chars[start..i].iter().collect();
            match items.last_mut() {
                Some(Item::Text(prev)) if !prev.chars().any(is_pinyin_char) => prev.push_str(&t),
                _ => items.push(Item::Text(t)),
            }
        }
    }
    items
}

// ---------------------------------------------------------------------------
// Pinyin
// ---------------------------------------------------------------------------

/// Every valid toneless pinyin syllable, used to segment unspaced input.
const SYLLABLES: &str = "a ai an ang ao \
ba bai ban bang bao bei ben beng bi bian biang biao bie bin bing bo bu \
ca cai can cang cao ce cei cen ceng ci cong cou cu cuan cui cun cuo \
cha chai chan chang chao che chen cheng chi chong chou chu chua chuai chuan chuang chui chun chuo \
da dai dan dang dao de dei den deng di dia dian diao die ding diu dong dou du duan dui dun duo \
e ei en eng er ê \
fa fan fang fei fen feng fiao fo fou fu \
ga gai gan gang gao ge gei gen geng gong gou gu gua guai guan guang gui gun guo \
ha hai han hang hao he hei hen heng hong hou hu hua huai huan huang hui hun huo hm hng \
ji jia jian jiang jiao jie jin jing jiong jiu ju juan jue jun \
ka kai kan kang kao ke kei ken keng kong kou ku kua kuai kuan kuang kui kun kuo \
la lai lan lang lao le lei leng li lia lian liang liao lie lin ling liu lo long lou lu luan lun luo lü lüe lue \
m ma mai man mang mao me mei men meng mi mian miao mie min ming miu mo mou mu \
n na nai nan nang nao ne nei nen neng ng ni nian niang niao nie nin ning niu nong nou nu nuan nun nuo nü nüe nue \
o ou \
pa pai pan pang pao pei pen peng pi pian piao pie pin ping po pou pu \
qi qia qian qiang qiao qie qin qing qiong qiu qu quan que qun \
ran rang rao re ren reng ri rong rou ru rua ruan rui run ruo \
sa sai san sang sao se sen seng si song sou su suan sui sun suo \
sha shai shan shang shao she shei shen sheng shi shou shu shua shuai shuan shuang shui shun shuo \
ta tai tan tang tao te tei teng ti tian tiao tie ting tong tou tu tuan tui tun tuo \
wa wai wan wang wei wen weng wo wu \
xi xia xian xiang xiao xie xin xing xiong xiu xu xuan xue xun \
ya yan yang yao ye yi yin ying yo yong you yu yuan yue yun \
za zai zan zang zao ze zei zen zeng zi zong zou zu zuan zui zun zuo \
zha zhai zhan zhang zhao zhe zhei zhen zheng zhi zhong zhou zhu zhua zhuai zhuan zhuang zhui zhun zhuo";

fn syllable_set() -> &'static HashSet<&'static str> {
    static SET: OnceLock<HashSet<&'static str>> = OnceLock::new();
    SET.get_or_init(|| SYLLABLES.split_whitespace().collect())
}

fn is_vowel_letter(c: char) -> bool {
    matches!(c, 'a' | 'e' | 'i' | 'o' | 'u' | 'ü' | 'ê' | 'v')
}

/// Can `cand` be one syllable, given the letter that follows it?
fn valid_syllable(cand: &str, next: Option<char>) -> bool {
    let set = syllable_set();
    if set.contains(cand) {
        return true;
    }
    // erhua: hua + r, but not when the r starts the next syllable (hua-ran)
    if let Some(stem) = cand.strip_suffix('r') {
        if stem != "e" && set.contains(stem) && !next.is_some_and(is_vowel_letter) {
            return true;
        }
    }
    false
}

#[derive(Clone, Copy)]
struct PChar {
    base: char,
    tone: Option<u8>,
    /// index into the original token chars, one past this letter
    src_end: usize,
}

/// Parse one run of Latin letters, e.g. `Zhōngguó`, `ni3hao3`, `xi'an`.
/// Returns `None` if the run is not pinyin (e.g. an English word).
pub fn parse_pinyin_token(token: &str) -> Option<Vec<(Syllable, String)>> {
    let src: Vec<char> = token.chars().collect();
    // Normalise into letters; split into segments at digits and apostrophes.
    let mut segments: Vec<(Vec<PChar>, Option<u8>)> = Vec::new();
    let mut cur: Vec<PChar> = Vec::new();
    for (idx, &c) in src.iter().enumerate() {
        if is_combining(c) {
            let last = cur.last_mut()?;
            match c {
                '\u{0304}' => last.tone = Some(1),
                '\u{0301}' => last.tone = Some(2),
                '\u{030C}' => last.tone = Some(3),
                '\u{0300}' => last.tone = Some(4),
                '\u{0308}' if last.base == 'u' => last.base = 'ü',
                '\u{0302}' if last.base == 'e' => last.base = 'ê',
                _ => {}
            }
            last.src_end = idx + 1;
        } else if let Some(d) = c.to_digit(10) {
            if d > 5 || cur.is_empty() {
                return None;
            }
            cur.last_mut()?.src_end = idx + 1;
            segments.push((std::mem::take(&mut cur), Some(d as u8)));
        } else if is_apostrophe(c) {
            if !cur.is_empty() {
                segments.push((std::mem::take(&mut cur), None));
            }
        } else if c == ':' {
            let last = cur.last_mut()?;
            if last.base == 'u' {
                last.base = 'ü';
                last.src_end = idx + 1;
            }
        } else {
            let (base, tone) = match toned_vowel(c) {
                Some((b, t)) => (b, Some(t)),
                None => {
                    let l = c.to_lowercase().next().unwrap_or(c);
                    let l = match l {
                        'v' => 'ü',
                        _ => l,
                    };
                    if !(l.is_ascii_lowercase() || l == 'ü' || l == 'ê') {
                        return None;
                    }
                    (l, None)
                }
            };
            cur.push(PChar { base, tone, src_end: idx + 1 });
        }
    }
    if !cur.is_empty() {
        segments.push((cur, None));
    }

    let mut out = Vec::new();
    let mut src_pos = 0;
    for (letters, digit) in segments {
        let spans = segment(&letters)?;
        let count = spans.len();
        for (k, (a, b)) in spans.into_iter().enumerate() {
            let text: String = letters[a..b].iter().map(|p| p.base).collect();
            let mut syl = Syllable::from_pinyin(&text)?;
            let marked = letters[a..b].iter().find_map(|p| p.tone);
            let tone = match (marked, digit) {
                (Some(t), _) => Tone::from_number(t),
                (None, Some(d)) if k + 1 == count => Tone::from_number(d),
                _ => Tone::Neutral,
            };
            syl.tone = tone;
            // src_end already covers any trailing tone digit
            let end = letters[b - 1].src_end;
            let source: String = src[src_pos..end]
                .iter()
                .filter(|c| !is_apostrophe(**c))
                .collect();
            src_pos = end;
            out.push((syl, source));
        }
    }
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

/// Longest-first segmentation with backtracking.
fn segment(letters: &[PChar]) -> Option<Vec<(usize, usize)>> {
    fn go(
        letters: &[PChar],
        bases: &[char],
        pos: usize,
        dead: &mut HashSet<usize>,
    ) -> Option<Vec<(usize, usize)>> {
        if pos == letters.len() {
            return Some(Vec::new());
        }
        if dead.contains(&pos) {
            return None;
        }
        let max = (letters.len() - pos).min(7);
        for len in (1..=max).rev() {
            let span = &letters[pos..pos + len];
            if span.iter().filter(|p| p.tone.is_some()).count() > 1 {
                continue;
            }
            let cand: String = bases[pos..pos + len].iter().collect();
            if !valid_syllable(&cand, bases.get(pos + len).copied()) {
                continue;
            }
            if let Some(mut rest) = go(letters, bases, pos + len, dead) {
                rest.insert(0, (pos, pos + len));
                return Some(rest);
            }
        }
        dead.insert(pos);
        None
    }
    let bases: Vec<char> = letters.iter().map(|p| p.base).collect();
    go(letters, &bases, 0, &mut HashSet::new())
}

// ---------------------------------------------------------------------------
// Zhuyin
// ---------------------------------------------------------------------------

#[derive(Default)]
struct ZhuyinAcc {
    initial: Option<char>,
    medial: Option<char>,
    rhyme: Option<char>,
    rhotic: bool,
    neutral: bool,
    source: String,
}

impl ZhuyinAcc {
    fn is_empty(&self) -> bool {
        self.initial.is_none() && self.medial.is_none() && self.rhyme.is_none()
    }
}

fn zhuyin_run(chars: &[char], items: &mut Vec<Item>) {
    let mut acc = ZhuyinAcc::default();
    let mut pending_neutral = false;

    fn flush(acc: &mut ZhuyinAcc, tone: Option<Tone>, items: &mut Vec<Item>) {
        if acc.is_empty() {
            if !acc.source.is_empty() {
                items.push(Item::Text(std::mem::take(&mut acc.source)));
            }
            *acc = ZhuyinAcc::default();
            return;
        }
        let tone = if acc.neutral {
            Tone::Neutral
        } else {
            tone.unwrap_or(Tone::First)
        };
        match zhuyin_syllable(acc) {
            Some(syl) => items.push(Item::Syl(SylItem {
                syl: syl.with_tone(tone),
                source: std::mem::take(&mut acc.source),
                origin: Origin::Zhuyin,
                sandhi_from: None,
            })),
            None => items.push(Item::Text(std::mem::take(&mut acc.source))),
        }
        *acc = ZhuyinAcc::default();
    }

    for &c in chars {
        let start_new = |acc: &mut ZhuyinAcc, pending: &mut bool| {
            if *pending {
                acc.neutral = true;
                *pending = false;
            }
        };
        match c {
            'ˉ' | 'ˊ' | 'ˇ' | 'ˋ' => {
                acc.source.push(c);
                let t = match c {
                    'ˉ' => Tone::First,
                    'ˊ' => Tone::Second,
                    'ˇ' => Tone::Third,
                    _ => Tone::Fourth,
                };
                flush(&mut acc, Some(t), items);
            }
            '˙' => {
                if acc.is_empty() {
                    pending_neutral = true;
                    acc.source.push(c);
                } else {
                    acc.source.push(c);
                    acc.neutral = true;
                    flush(&mut acc, None, items);
                }
            }
            _ if Consonant::from_zhuyin(c).is_some() => {
                if !acc.is_empty() {
                    flush(&mut acc, None, items);
                }
                start_new(&mut acc, &mut pending_neutral);
                acc.initial = Some(c);
                acc.source.push(c);
            }
            'ㄧ' | 'ㄨ' | 'ㄩ' => {
                if acc.medial.is_some() || acc.rhyme.is_some() {
                    flush(&mut acc, None, items);
                }
                start_new(&mut acc, &mut pending_neutral);
                acc.medial = Some(c);
                acc.source.push(c);
            }
            'ㄦ' if !acc.is_empty() => {
                acc.rhotic = true;
                acc.source.push(c);
            }
            _ => {
                if acc.rhyme.is_some() {
                    flush(&mut acc, None, items);
                }
                start_new(&mut acc, &mut pending_neutral);
                acc.rhyme = Some(c);
                acc.source.push(c);
            }
        }
    }
    flush(&mut acc, None, items);
}

fn zhuyin_syllable(acc: &ZhuyinAcc) -> Option<Syllable> {
    let initial = match acc.initial {
        Some(c) => Some(Consonant::from_zhuyin(c)?),
        None => None,
    };
    let rhyme = match acc.rhyme {
        None => "",
        Some(c) => match c {
            'ㄚ' => "a",
            'ㄛ' => "o",
            'ㄜ' => "e",
            'ㄝ' => "ê",
            'ㄞ' => "ai",
            'ㄟ' => "ei",
            'ㄠ' => "ao",
            'ㄡ' => "ou",
            'ㄢ' => "an",
            'ㄣ' => "en",
            'ㄤ' => "ang",
            'ㄥ' => "eng",
            'ㄦ' => "er",
            _ => return None,
        },
    };
    let medial = match acc.medial {
        None => "",
        Some('ㄧ') => "i",
        Some('ㄨ') => "u",
        _ => "ü",
    };
    let fin: String = match (medial, rhyme) {
        ("", "") => {
            // zhi chi shi ri zi ci si written with the initial alone
            let mut syl = Syllable::empty();
            syl.initial = initial;
            syl.rhotic = acc.rhotic;
            return initial.filter(|c| c.takes_apical_vowel()).map(|_| syl);
        }
        ("", "er") => {
            let mut syl = Syllable::from_pinyin("er")?;
            syl.initial = initial;
            return Some(syl);
        }
        ("i", "en") => "in".into(),
        ("i", "eng") => "ing".into(),
        ("i", "ê") => "ie".into(),
        ("i", "ou") => "iou".into(),
        ("u", "ei") => "uei".into(),
        ("u", "en") => "uen".into(),
        ("u", "eng") if initial.is_some() => "ong".into(),
        ("u", "eng") => "ueng".into(),
        ("ü", "ê") => "üe".into(),
        ("ü", "en") => "ün".into(),
        ("ü", "eng") => "iong".into(),
        (m, r) => format!("{m}{r}"),
    };
    // Build via the pinyin analyser, using a dummy-free route: spell with the
    // initial's pinyin, but keep ü explicit so j/q/x rules don't interfere.
    let (_, med, nuc, coda) = crate::phonology::FINALS.iter().find(|e| e.0 == fin)?;
    Some(Syllable {
        initial,
        medial: *med,
        nucleus: Some(*nuc),
        coda: *coda,
        rhotic: acc.rhotic,
        tone: Tone::Neutral,
    })
}

// ---------------------------------------------------------------------------
// Hanzi
// ---------------------------------------------------------------------------

/// Common words whose characters have a context-dependent reading that the
/// per-character default gets wrong. Tones are citation tones (no sandhi).
const PHRASES: &[(&str, &str)] = &[
    ("银行", "yin2 hang2"), ("行业", "hang2 ye4"), ("行长", "hang2 zhang3"),
    ("长大", "zhang3 da4"), ("校长", "xiao4 zhang3"), ("成长", "cheng2 zhang3"),
    ("增长", "zeng1 zhang3"), ("部长", "bu4 zhang3"), ("市长", "shi4 zhang3"),
    ("家长", "jia1 zhang3"), ("班长", "ban1 zhang3"), ("生长", "sheng1 zhang3"),
    ("重新", "chong2 xin1"), ("重复", "chong2 fu4"),
    ("还钱", "huan2 qian2"), ("归还", "gui1 huan2"),
    ("首都", "shou3 du1"), ("都市", "du1 shi4"),
    ("爱好", "ai4 hao4"), ("好奇", "hao4 qi2"), ("好客", "hao4 ke4"),
    ("睡觉", "shui4 jiao4"), ("午觉", "wu3 jiao4"), ("觉得", "jue2 de5"),
    ("记得", "ji4 de5"), ("懂得", "dong3 de5"), ("值得", "zhi2 de5"),
    ("头发", "tou2 fa5"), ("理发", "li3 fa4"),
    ("音乐", "yin1 yue4"), ("乐器", "yue4 qi4"),
    ("成为", "cheng2 wei2"), ("作为", "zuo4 wei2"), ("认为", "ren4 wei2"),
    ("以为", "yi3 wei2"), ("行为", "xing2 wei2"), ("为了", "wei4 le5"),
    ("一只", "yi1 zhi1"), ("两只", "liang3 zhi1"),
    ("种子", "zhong3 zi5"), ("种植", "zhong4 zhi2"), ("种地", "zhong4 di4"),
    ("着急", "zhao2 ji2"), ("睡着", "shui4 zhao2"), ("着火", "zhao2 huo3"),
    ("地方", "di4 fang5"), ("东西", "dong1 xi5"),
    ("朝阳", "zhao1 yang2"), ("干净", "gan1 jing4"), ("干杯", "gan1 bei1"),
    ("几乎", "ji1 hu1"), ("传记", "zhuan4 ji4"), ("会计", "kuai4 ji4"),
    ("便宜", "pian2 yi5"), ("教书", "jiao1 shu1"), ("关系", "guan1 xi5"),
    ("什么", "shen2 me5"), ("怎么", "zen3 me5"), ("那么", "na4 me5"),
    ("这么", "zhe4 me5"), ("多么", "duo1 me5"), ("时候", "shi2 hou5"),
    ("朋友", "peng2 you5"), ("谢谢", "xie4 xie5"), ("先生", "xian1 sheng5"),
    ("喜欢", "xi3 huan5"), ("认识", "ren4 shi5"), ("知道", "zhi1 dao5"),
    ("明白", "ming2 bai5"), ("漂亮", "piao4 liang5"), ("衣服", "yi1 fu5"),
    ("休息", "xiu1 xi5"), ("意思", "yi4 si5"), ("事情", "shi4 qing5"),
    ("舒服", "shu1 fu5"), ("告诉", "gao4 su5"), ("麻烦", "ma2 fan5"),
    ("清楚", "qing1 chu5"), ("大夫", "dai4 fu5"), ("了解", "liao3 jie3"),
    ("目的", "mu4 di4"), ("的确", "di2 que4"), ("中奖", "zhong4 jiang3"),
    ("少年", "shao4 nian2"), ("妈妈", "ma1 ma5"), ("爸爸", "ba4 ba5"),
    ("哥哥", "ge1 ge5"), ("姐姐", "jie3 jie5"), ("弟弟", "di4 di5"),
    ("妹妹", "mei4 mei5"), ("奶奶", "nai3 nai5"), ("爷爷", "ye2 ye5"),
    ("孩子", "hai2 zi5"), ("儿子", "er2 zi5"), ("桌子", "zhuo1 zi5"),
    ("椅子", "yi3 zi5"), ("房子", "fang2 zi5"), ("日子", "ri4 zi5"),
    ("样子", "yang4 zi5"), ("杯子", "bei1 zi5"), ("筷子", "kuai4 zi5"),
    ("电子", "dian4 zi3"), ("原子", "yuan2 zi3"), ("分子", "fen1 zi3"),
    ("孔子", "kong3 zi3"), ("老子", "lao3 zi3"), ("君子", "jun1 zi3"),
    ("女子", "nü3 zi3"), ("男子", "nan2 zi3"), ("子女", "zi3 nü3"),
    ("长城", "chang2 cheng2"), ("调查", "diao4 cha2"), ("差不多", "cha4 bu4 duo1"),
    ("出差", "chu1 chai1"), ("还是", "hai2 shi4"), ("数学", "shu4 xue2"),
    ("觉悟", "jue2 wu4"), ("得到", "de2 dao4"), ("看守", "kan1 shou3"),
    ("一会儿", "yi1 hui4 r"), ("哪儿", "na3 r"), ("这儿", "zhe4 r"), ("那儿", "na4 r"),
    ("一点儿", "yi1 dian3 r"), ("玩儿", "wan2 r"),
];

/// Per-character readings where the dictionary default is the rarer one.
const CHAR_DEFAULTS: &[(char, &str)] = &[
    ('长', "chang2"),
    ('子', "zi3"),
    ('地', "di4"),
    ('得', "de5"),
    ('谁', "shei2"),
    ('嗯', "ng5"),
];

fn reading_syllable(reading: &str) -> Option<Syllable> {
    parse_pinyin_token(reading).and_then(|v| v.into_iter().next().map(|(s, _)| s))
}

fn hanzi_run(chars: &[char], items: &mut Vec<Item>) {
    use pinyin::ToPinyin;
    let mut i = 0;
    while i < chars.len() {
        // longest phrase match
        let mut matched = false;
        for len in (2..=4).rev() {
            if i + len > chars.len() {
                continue;
            }
            let word: String = chars[i..i + len].iter().collect();
            if let Some((_, reading)) = PHRASES.iter().find(|p| p.0 == word) {
                let readings: Vec<&str> = reading.split_whitespace().collect();
                // "r" entries attach erhua to the previous syllable
                for (k, r) in readings.into_iter().enumerate() {
                    if r == "r" {
                        if let Some(Item::Syl(prev)) = items.last_mut() {
                            prev.syl.rhotic = true;
                            prev.source.push(chars[i + k]);
                        }
                    } else if let Some(syl) = reading_syllable(r) {
                        items.push(Item::Syl(SylItem {
                            syl,
                            source: chars[i + k].to_string(),
                            origin: Origin::Hanzi,
                            sandhi_from: None,
                        }));
                    }
                }
                i += len;
                matched = true;
                break;
            }
        }
        if matched {
            continue;
        }
        let c = chars[i];
        let syl = if let Some((_, r)) = CHAR_DEFAULTS.iter().find(|e| e.0 == c) {
            // 子 is a neutral-tone suffix after another character (桌子, 孩子 …)
            if c == '子' && i > 0 {
                reading_syllable("zi5")
            } else {
                reading_syllable(r)
            }
        } else {
            c.to_pinyin().and_then(|p| reading_syllable(p.with_tone()))
        };
        match syl {
            Some(syl) => items.push(Item::Syl(SylItem {
                syl,
                source: c.to_string(),
                origin: Origin::Hanzi,
                sandhi_from: None,
            })),
            None => items.push(Item::Text(c.to_string())),
        }
        i += 1;
    }
}

// ---------------------------------------------------------------------------
// Tone sandhi (optional display mode)
// ---------------------------------------------------------------------------

/// Apply the regular Mandarin tone sandhi rules within each phrase
/// (a run of syllables not broken by punctuation or a line break).
/// Citation tones are kept in `sandhi_from`.
pub fn apply_sandhi(items: &mut [Item]) {
    let mut runs: Vec<Vec<usize>> = vec![Vec::new()];
    for (i, item) in items.iter().enumerate() {
        match item {
            Item::Syl(_) => runs.last_mut().unwrap().push(i),
            Item::Space => {}
            _ => runs.push(Vec::new()),
        }
    }
    for run in runs {
        let tones: Vec<Tone> = run
            .iter()
            .map(|&i| match &items[i] {
                Item::Syl(s) => s.syl.tone,
                _ => Tone::Neutral,
            })
            .collect();
        let mut new = tones.clone();
        for k in 0..run.len() {
            let next = tones.get(k + 1).copied();
            let Item::Syl(s) = &items[run[k]] else { continue };
            let is_char = |c: &str| s.origin == Origin::Hanzi && s.source == c;
            if is_char("一") && tones[k] == Tone::First {
                match next {
                    Some(Tone::Fourth) => new[k] = Tone::Second,
                    Some(Tone::First | Tone::Second | Tone::Third) => new[k] = Tone::Fourth,
                    _ => {}
                }
            } else if is_char("不") && tones[k] == Tone::Fourth {
                if next == Some(Tone::Fourth) {
                    new[k] = Tone::Second;
                }
            } else if tones[k] == Tone::Third && next == Some(Tone::Third) {
                new[k] = Tone::Second;
            }
        }
        for (k, &i) in run.iter().enumerate() {
            if let Item::Syl(s) = &mut items[i] {
                if new[k] != s.syl.tone {
                    s.sandhi_from = Some(s.syl.tone);
                    s.syl.tone = new[k];
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pinyin_of(input: &str) -> Vec<String> {
        convert(input)
            .into_iter()
            .filter_map(|i| match i {
                Item::Syl(s) => Some(s.syl.pinyin()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn pinyin_with_marks_and_no_spaces() {
        assert_eq!(pinyin_of("nǐhǎo"), ["nǐ", "hǎo"]);
        assert_eq!(pinyin_of("Zhōngguó"), ["zhōng", "guó"]);
        assert_eq!(pinyin_of("Xī'ān"), ["xī", "ān"]);
        assert_eq!(pinyin_of("xīān"), ["xī", "ān"]);
        assert_eq!(pinyin_of("xiān"), ["xiān"]);
        assert_eq!(pinyin_of("fāng'àn"), ["fāng", "àn"]);
    }

    #[test]
    fn pinyin_with_numbers() {
        assert_eq!(pinyin_of("ni3 hao3"), ["nǐ", "hǎo"]);
        assert_eq!(pinyin_of("ni3hao3"), ["nǐ", "hǎo"]);
        assert_eq!(pinyin_of("lv4 nu:3"), ["lǜ", "nǚ"]);
        assert_eq!(pinyin_of("ma5"), ["ma"]);
    }

    #[test]
    fn erhua() {
        assert_eq!(pinyin_of("huār"), ["huār"]);
        assert_eq!(pinyin_of("yìdiǎnr"), ["yì", "diǎnr"]);
        assert_eq!(pinyin_of("huàrán"), ["huà", "rán"]);
    }

    #[test]
    fn english_is_left_alone() {
        let items = convert("hello");
        assert!(matches!(&items[0], Item::Text(t) if t == "hello"));
    }

    #[test]
    fn zhuyin() {
        assert_eq!(pinyin_of("ㄋㄧˇ ㄏㄠˇ"), ["nǐ", "hǎo"]);
        assert_eq!(pinyin_of("ㄓㄨㄥㄍㄨㄛˊ"), ["zhōng", "guó"]);
        assert_eq!(pinyin_of("ㄒㄩㄝˊ ㄕˋ ˙ㄉㄜ"), ["xué", "shì", "de"]);
        assert_eq!(pinyin_of("ㄩㄥˇ"), ["yǒng"]);
        assert_eq!(pinyin_of("ㄦˊ"), ["ér"]);
    }

    #[test]
    fn hanzi() {
        assert_eq!(pinyin_of("你好"), ["nǐ", "hǎo"]);
        assert_eq!(pinyin_of("银行"), ["yín", "háng"]);
        assert_eq!(pinyin_of("孩子"), ["hái", "zi"]);
        assert_eq!(pinyin_of("一点儿"), ["yī", "diǎnr"]);
    }

    #[test]
    fn sandhi() {
        let mut items = convert("你好 一个 不是");
        apply_sandhi(&mut items);
        let tones: Vec<String> = items
            .iter()
            .filter_map(|i| match i {
                Item::Syl(s) => Some(s.syl.pinyin()),
                _ => None,
            })
            .collect();
        assert_eq!(tones, ["ní", "hǎo", "yí", "gè", "bú", "shì"]);
    }
}
