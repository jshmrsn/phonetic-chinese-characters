mod brush;
mod hanzi;
mod keyboard;
mod glyph;
mod parse;
mod phonology;
mod segment;

use brush::Brush;
use dioxus::prelude::*;
use glyph::{Layer, Part, Role};
use parse::{Item, SylItem};
use phonology::{Coda, Consonant, Manner, Place, Syllable, Tone, Vowel, R_IDEA, R_STROKE};

const CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::launch(App);
}

const EXAMPLES: &[(&str, &str)] = &[
    ("Mixed", "你好，世界！\nnǐ hǎo, shìjiè!\nㄋㄧˇ ㄏㄠˇ，ㄕˋ ㄐㄧㄝˋ！"),
    (
        "新闻",
        "本市新建的中央图书馆今天上午正式向市民开放。图书馆共有五层，藏书超过八十万册，还设有儿童阅读区、自习室和咖啡厅。开馆第一天，不少家长带着孩子早早来到门口排队，现场气氛十分热闹。\n馆长在接受记者采访时表示，希望图书馆能成为大家安静学习、交流想法的地方。今后，馆里还将定期举办讲座和展览，周末的开放时间也会延长到晚上九点。",
    ),
    ("静夜思", "床前明月光，疑是地上霜。\n举头望明月，低头思故乡。"),
    ("Pinyin", "Wǒ xiǎng hē yì bēi chá. Zhōngguó de chá hěn yǒumíng."),
    ("Tone numbers", "wo3 xiang3 qu4 Bei3jing1 xue2 zhong1wen2"),
    ("Zhuyin", "ㄨㄛˇ ㄒㄧˇ ㄏㄨㄢ ㄓㄨㄥ ㄨㄣˊ"),
    ("儿化", "一点儿 一会儿 玩儿 huār zhèr"),
    ("Sandhi", "你好！一个，一天，不是，不对，很好。"),
];

/// What to print under each block.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Captions {
    Off,
    Pinyin,
    Zhuyin,
    Both,
}

impl Captions {
    const ALL: [Captions; 4] = [Captions::Zhuyin, Captions::Pinyin, Captions::Both, Captions::Off];

    fn key(self) -> &'static str {
        match self {
            Captions::Off => "off",
            Captions::Pinyin => "pinyin",
            Captions::Zhuyin => "zhuyin",
            Captions::Both => "both",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Captions::Off => "No captions",
            Captions::Pinyin => "Pinyin captions",
            Captions::Zhuyin => "Zhuyin captions",
            Captions::Both => "Zhuyin + pinyin",
        }
    }

    fn from_key(k: &str) -> Captions {
        Captions::ALL.into_iter().find(|c| c.key() == k).unwrap_or(Captions::Zhuyin)
    }
}

/// Commas and full stops sit at the bottom of the line of blocks, as in
/// printed Chinese, instead of in the middle.
fn sits_low(text: &str) -> bool {
    !text.is_empty() && text.chars().all(|c| matches!(c, '，' | '。' | '、' | '．' | '｡' | '､' | ',' | '.'))
}

fn syl(p: &str) -> Syllable {
    parse::parse_pinyin_token(p)
        .and_then(|v| v.into_iter().next())
        .map(|(s, _)| s)
        .unwrap_or_else(Syllable::empty)
}

/// Colour theme: follow the system, or force day or night.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Theme {
    Auto,
    Day,
    Night,
}

/// A run of output kept together on one line: a word's blocks, then any
/// punctuation and spaces that follow it.
struct WordGroup {
    items: Vec<usize>,
    /// the segmenter's word number, for hanzi
    word: Option<u32>,
    newline: bool,
    /// how many of the leading items are the word's blocks
    blocks: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Tab {
    Text,
    Keyboard,
}

#[component]
fn App() -> Element {
    let mut input = use_signal(|| {
        // start with the two-paragraph news article
        EXAMPLES.iter().find(|e| e.0 == "新闻").unwrap_or(&EXAMPLES[0]).1.to_string()
    });
    let size = use_signal(|| 40u32);
    let captions = use_signal(|| Captions::Off);
    let colorize = use_signal(|| false);
    let sandhi = use_signal(|| false);
    // word boundaries are shown with bridges at tone-mark height by default
    let word_spaces = use_signal(|| false);
    let word_underlines = use_signal(|| false);
    let word_bridges = use_signal(|| true);
    let word_dots = use_signal(|| false);
    let word_lines = use_signal(|| false);
    // bridge length as a share of a block width
    let bridge_len = use_signal(|| 0.35f32);
    // vertical line length as a share of a block height
    let line_len = use_signal(|| 0.3f32);
    // bridges and dots at the blocks' vertical centre instead of tone-mark height
    let bridge_mid = use_signal(|| false);
    let theme = use_signal(|| Theme::Auto);
    // the stylesheet keys its colours off <html data-theme>
    use_effect(move || {
        let value = match theme() {
            Theme::Auto => "",
            Theme::Day => "light",
            Theme::Night => "dark",
        };
        document::eval(&format!(
            "const r = document.documentElement; if ('{value}') {{ r.dataset.theme = '{value}'; }} else {{ delete r.dataset.theme; }}"
        ));
    });
    let mut selected = use_signal(|| None::<usize>);
    use_context_provider(|| Signal::new(Brush::PEN));
    let mut tab = use_signal(|| Tab::Text);

    let items = use_memo(move || {
        let opts = parse::Options { word_spaces: word_spaces() };
        let mut items = parse::convert_with(&input.read(), opts);
        if sandhi() {
            parse::apply_sandhi(&mut items);
        }
        items
    });

    let selected_item: Option<SylItem> = selected().and_then(|i| match items.read().get(i) {
        Some(Item::Syl(s)) => Some(s.clone()),
        _ => None,
    });
    let syllable_count = items.read().iter().filter(|i| matches!(i, Item::Syl(_))).count();
    let wide = parse::wide_spaces(&items.read().iter().map(Item::flow).collect::<Vec<_>>());
    // Group each word (by spaces, and by the segmenter's word numbers) with
    // the punctuation and spaces after it, so lines break between words and
    // never start with a gap. A line break is a group of its own.
    let groups: Vec<WordGroup> = {
        let items = items.read();
        let mut groups: Vec<WordGroup> = Vec::new();
        let mut prev_space = false;
        for (i, item) in items.iter().enumerate() {
            let space = matches!(item, Item::Space { .. });
            let (is_syl, word) = match item {
                Item::Syl(s) => (true, s.word),
                _ => (false, None),
            };
            let start_new = match groups.last() {
                None => true,
                Some(g) => {
                    matches!(item, Item::Newline)
                        || g.newline
                        || (prev_space && !space)
                        || (is_syl && word != g.word)
                }
            };
            if start_new {
                groups.push(WordGroup {
                    items: vec![i],
                    word,
                    newline: matches!(item, Item::Newline),
                    blocks: usize::from(is_syl),
                });
            } else {
                let g = groups.last_mut().unwrap();
                g.items.push(i);
                // the blocks of a word come first; count them for the underline
                if is_syl && g.blocks == g.items.len() - 1 {
                    g.blocks += 1;
                }
            }
            prev_space = space;
        }
        groups
    };
    let pane = |t: Tab| if tab() == t { "tab-pane" } else { "tab-pane inactive" };

    rsx! {
        document::Title { "Phonetic Chinese Characters Brainstorming" }
        document::Stylesheet { href: CSS }
        div { class: "page",
            header { class: "masthead",
                div { class: "logo colorize",
                    Glyph { syl: syl("zhōng"), size: 54 }
                    Glyph { syl: syl("wén"), size: 54 }
                }
                div { class: "title",
                    h1 { "Phonetic Chinese Characters Brainstorming" }
                    p { "An experiment in writing Mandarin phonetically, one square block per syllable, using the strokes of Chinese characters." }
                }
            }

            nav { class: "tabs", role: "tablist",
                button {
                    class: if tab() == Tab::Text { "tab active" } else { "tab" },
                    role: "tab",
                    onclick: move |_| tab.set(Tab::Text),
                    span { class: "tab-zh", "文 → 块" }
                    "Text"
                }
                button {
                    class: if tab() == Tab::Keyboard { "tab active" } else { "tab" },
                    role: "tab",
                    onclick: move |_| tab.set(Tab::Keyboard),
                    span { class: "tab-zh", "⌨" }
                    "Keyboard"
                }
            }

            DisplayBar {
                size,
                captions,
                colorize,
                sandhi,
                word_spaces,
                word_underlines,
                word_bridges,
                word_dots,
                word_lines,
                bridge_len,
                line_len,
                bridge_mid,
                theme,
            }

            // both tabs stay mounted, so switching never loses what was typed
            div { class: pane(Tab::Text),
                section { class: "panel input-panel",
                    label { r#for: "src", "Type or paste Hanzi, pinyin (marks or numbers) or zhuyin" }
                    textarea {
                        id: "src",
                        spellcheck: "false",
                        rows: "4",
                        value: "{input}",
                        oninput: move |e| {
                            input.set(e.value());
                            selected.set(None);
                        },
                    }
                    div { class: "examples",
                        span { class: "muted", "Examples:" }
                        for (name , text) in EXAMPLES.iter() {
                            button {
                                class: "chip",
                                onclick: move |_| {
                                    input.set(text.to_string());
                                    selected.set(None);
                                },
                                "{name}"
                            }
                        }
                    }
                }
                div { class: "workspace",
                    section { class: if colorize() { "panel output-panel colorize" } else { "panel output-panel" },
                        div { class: "output-head",
                            span { class: "muted", "{syllable_count} syllables" }
                        }
                        div { class: "output",
                            for group in groups {
                                if group.newline {
                                    div { class: "break" }
                                } else {
                                    span { class: "word",
                                        // multi-block words can carry an inset underline
                                        span {
                                            class: if word_underlines() && group.word.is_some() && group.blocks >= 2 { "word-blocks underlined" } else { "word-blocks" },
                                            style: "--inset: {size() / 3}px;",
                                            if word_underlines() && group.word.is_some() && group.blocks >= 2 {
                                                Underline { blocks: group.blocks, size: size(), seed: group.items[0] as u32 }
                                            }
                                            if word_bridges() && group.word.is_some() && group.blocks >= 2 {
                                                Bridges {
                                                    blocks: group.blocks,
                                                    size: size(),
                                                    seed: group.items[0] as u32,
                                                    kind: BridgeKind::Bar,
                                                    length: bridge_len(),
                                                    middle: bridge_mid(),
                                                }
                                            }
                                            if word_dots() && group.word.is_some() && group.blocks >= 2 {
                                                Bridges {
                                                    blocks: group.blocks,
                                                    size: size(),
                                                    seed: group.items[0] as u32,
                                                    kind: BridgeKind::Dot,
                                                    middle: bridge_mid(),
                                                }
                                            }
                                            if word_lines() && group.word.is_some() && group.blocks >= 2 {
                                                Bridges {
                                                    blocks: group.blocks,
                                                    size: size(),
                                                    seed: group.items[0] as u32,
                                                    kind: BridgeKind::Line,
                                                    length: line_len(),
                                                    middle: bridge_mid(),
                                                }
                                            }
                                            for i in group.items[..group.blocks].iter().copied() {
                                                ItemView {
                                                    key: "{i}",
                                                    item: items.read()[i].clone(),
                                                    index: i,
                                                    wide: wide[i],
                                                    size: size(),
                                                    captions: captions(),
                                                    selected: selected() == Some(i),
                                                    on_select: move |i| selected.set(Some(i)),
                                                }
                                            }
                                        }
                                        for i in group.items[group.blocks..].iter().copied() {
                                            ItemView {
                                                key: "{i}",
                                                item: items.read()[i].clone(),
                                                index: i,
                                                wide: wide[i],
                                                size: size(),
                                                captions: captions(),
                                                selected: selected() == Some(i),
                                                on_select: move |i| selected.set(Some(i)),
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Inspector { item: selected_item }
                }
            }

            div { class: pane(Tab::Keyboard),
                KeyboardTab {
                    input,
                    size: size(),
                    captions: captions(),
                    colorize: colorize(),
                }
            }

            Reference {}

            footer { class: "footer",
                p {
                    "Hanzi are read with the per-character dictionary of the "
                    code { "pinyin" }
                    " crate plus a small table of common context-dependent words, so some polyphonic characters (行, 长, 得 …) can come out with the wrong reading. Pinyin and zhuyin input are exact."
                }
            }
        }
    }
}

/// Settings shared by both tabs: captions, colour, size and the brush.
#[component]
fn DisplayBar(
    mut size: Signal<u32>,
    mut captions: Signal<Captions>,
    mut colorize: Signal<bool>,
    mut sandhi: Signal<bool>,
    mut word_spaces: Signal<bool>,
    mut word_underlines: Signal<bool>,
    mut word_bridges: Signal<bool>,
    mut word_dots: Signal<bool>,
    mut word_lines: Signal<bool>,
    mut bridge_len: Signal<f32>,
    mut line_len: Signal<f32>,
    mut bridge_mid: Signal<bool>,
    mut theme: Signal<Theme>,
) -> Element {
    let mut brush = use_context::<Signal<Brush>>();
    let mut hand = use_signal(|| 0.5f32);
    rsx! {
        section { class: "panel display-panel",
            div { class: "toggles",
                select {
                    class: "caption-select",
                    "aria-label": "Captions",
                    value: "{captions().key()}",
                    onchange: move |e| captions.set(Captions::from_key(&e.value())),
                    for c in Captions::ALL {
                        option { value: "{c.key()}", selected: captions() == c, "{c.label()}" }
                    }
                }
                label { class: "toggle",
                    input {
                        r#type: "checkbox",
                        checked: colorize(),
                        onchange: move |e| colorize.set(e.checked()),
                    }
                    "Colour by part"
                }
                label { class: "toggle",
                    title: "Show spoken tones in the Text tab (3-3 → 2-3, 一 and 不 changes). Changed marks turn orange.",
                    input {
                        r#type: "checkbox",
                        checked: sandhi(),
                        onchange: move |e| sandhi.set(e.checked()),
                    }
                    "Tone sandhi"
                }
                label { class: "toggle",
                    title: "In the Text tab, put a space between the words of Chinese text (best guess from a word-frequency dictionary)",
                    input {
                        r#type: "checkbox",
                        checked: word_spaces(),
                        onchange: move |e| word_spaces.set(e.checked()),
                    }
                    "Word spaces"
                }
                label { class: "toggle",
                    title: "In the Text tab, underline each word of two or more blocks (same best-guess words as Word spaces)",
                    input {
                        r#type: "checkbox",
                        checked: word_underlines(),
                        onchange: move |e| word_underlines.set(e.checked()),
                    }
                    "Word underlines"
                }
                label { class: "toggle",
                    title: "In the Text tab, link the blocks of each word with a short stroke",
                    input {
                        r#type: "checkbox",
                        checked: word_bridges(),
                        onchange: move |e| word_bridges.set(e.checked()),
                    }
                    "Word bridges"
                }
                if word_bridges() {
                    label { class: "toggle size",
                        title: "Bridge length, as a share of a block width",
                        "Bridge length"
                        input {
                            r#type: "range",
                            min: "10",
                            max: "90",
                            step: "5",
                            value: "{(bridge_len() * 100.0).round()}",
                            oninput: move |e| bridge_len.set(e.value().parse::<f32>().unwrap_or(35.0) / 100.0),
                        }
                    }
                }
                label { class: "toggle",
                    title: "In the Text tab, put a small dot between the blocks of each word",
                    input {
                        r#type: "checkbox",
                        checked: word_dots(),
                        onchange: move |e| word_dots.set(e.checked()),
                    }
                    "Word dots"
                }
                label { class: "toggle",
                    title: "In the Text tab, put a short vertical line between the blocks of each word",
                    input {
                        r#type: "checkbox",
                        checked: word_lines(),
                        onchange: move |e| word_lines.set(e.checked()),
                    }
                    "Word lines"
                }
                if word_lines() {
                    label { class: "toggle size",
                        title: "Line length, as a share of a block height",
                        "Line length"
                        input {
                            r#type: "range",
                            min: "10",
                            max: "100",
                            step: "5",
                            value: "{(line_len() * 100.0).round()}",
                            oninput: move |e| line_len.set(e.value().parse::<f32>().unwrap_or(30.0) / 100.0),
                        }
                    }
                }
                if word_bridges() || word_dots() || word_lines() {
                    div { class: "segmented", role: "group", "aria-label": "Bridge, dot and line height",
                        for (mid , label , hint) in [
                            (false, "Top", "bridges and dots at the height of the tone marks; lines hang from the top of the blocks"),
                            (true, "Middle", "bridges, dots and lines at the vertical centre of the blocks"),
                        ]
                        {
                            button {
                                class: if bridge_mid() == mid { "seg active" } else { "seg" },
                                title: "{hint}",
                                onclick: move |_| bridge_mid.set(mid),
                                "{label}"
                            }
                        }
                    }
                }
                label { class: "toggle size",
                    "Size"
                    input {
                        r#type: "range",
                        min: "20",
                        max: "160",
                        value: "{size}",
                        oninput: move |e| size.set(e.value().parse().unwrap_or(40)),
                    }
                }
                div { class: "segmented", role: "group", "aria-label": "Colour theme",
                    for (t , label , hint) in [
                        (Theme::Auto, "Auto", "follow the system setting"),
                        (Theme::Day, "☀ Day", "light colours"),
                        (Theme::Night, "☾ Night", "dark colours"),
                    ]
                    {
                        button {
                            class: if theme() == t { "seg active" } else { "seg" },
                            title: "{hint}",
                            onclick: move |_| theme.set(t),
                            "{label}"
                        }
                    }
                }
                span { class: "divider" }
                span { class: "brush-title", "Brush" }
                for (name , preset , h) in [("Geometric", Brush::GEOMETRIC, 0.0f32), ("Pen", Brush::PEN, 0.5), ("Brush", Brush::BRUSH, 1.0)] {
                    button {
                        class: if brush() == preset { "chip active" } else { "chip" },
                        onclick: move |_| {
                            brush.set(preset);
                            hand.set(h);
                        },
                        "{name}"
                    }
                }
                label { class: "toggle hand",
                    span { class: "muted", "geometric" }
                    input {
                        r#type: "range",
                        min: "0",
                        max: "100",
                        value: "{(hand() * 100.0).round()}",
                        oninput: move |e| {
                            let v = e.value().parse::<f32>().unwrap_or(50.0) / 100.0;
                            hand.set(v);
                            brush.set(Brush::hand(v));
                        },
                    }
                    span { class: "muted", "handwritten" }
                }
            }
            details { class: "knobs",
                summary { "Fine-tune the brush" }
                div { class: "knob-grid",
                    Knob {
                        label: "Weight",
                        hint: "overall stroke thickness",
                        value: brush().weight,
                        min: 0.6,
                        max: 1.5,
                        on_change: move |v| brush.with_mut(|b| b.weight = v),
                    }
                    Knob {
                        label: "Pressure",
                        hint: "thick–thin contrast: slim waists, heavy verticals, swelling 捺 feet",
                        value: brush().pressure,
                        on_change: move |v| brush.with_mut(|b| b.pressure = v),
                    }
                    Knob {
                        label: "Taper",
                        hint: "pointed tips on 撇, 提 and hooks; teardrop 点",
                        value: brush().taper,
                        on_change: move |v| brush.with_mut(|b| b.taper = v),
                    }
                    Knob {
                        label: "Pauses 顿笔",
                        hint: "pressed heads and tails on 横 and 竖, pressed corners on 折",
                        value: brush().pause,
                        on_change: move |v| brush.with_mut(|b| b.pause = v),
                    }
                    Knob {
                        label: "Arcs",
                        hint: "bowed strokes, deeper curves, horizontals rising to the right",
                        value: brush().arc,
                        on_change: move |v| brush.with_mut(|b| b.arc = v),
                    }
                    Knob {
                        label: "Hooks 钩",
                        hint: "how far hooks flick out",
                        value: brush().hook,
                        on_change: move |v| brush.with_mut(|b| b.hook = v),
                    }
                    Knob {
                        label: "Wobble",
                        hint: "an unsteady hand",
                        value: brush().wobble,
                        on_change: move |v| brush.with_mut(|b| b.wobble = v),
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Keyboard tab: an editable phonetic area, the inspector and the keyboard
// ---------------------------------------------------------------------------

/// What the keyboard has written so far.
#[derive(Clone, Debug, PartialEq)]
enum Written {
    Syl(Syllable),
    /// digits, punctuation, emoji, English letters
    Text(String),
    Space,
    Newline,
}

/// Which board the keyboard shows.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Board {
    Letters,
    Latin,
    Numbers,
    Emoji,
}

/// What to print under each key.
#[derive(Clone, Copy, Debug, PartialEq)]
enum KeyLabels {
    Zhuyin,
    Pinyin,
    Both,
}

/// The editable phonetic area: what has been written, where the caret is,
/// which block the inspector shows, and the block being typed.
#[derive(Clone, Copy, PartialEq)]
struct Pad {
    written: Signal<Vec<Written>>,
    /// insertion point, 0..=written.len()
    cursor: Signal<usize>,
    selected: Signal<Option<usize>>,
    typed: Signal<Vec<keyboard::Key>>,
    rhotic: Signal<bool>,
}

impl Pad {
    fn caret(&self) -> usize {
        (self.cursor)().min(self.written.read().len())
    }

    /// Insert at the caret and move the caret past it.
    fn insert(mut self, w: Written) {
        let c = self.caret();
        self.written.write().insert(c, w);
        self.cursor.set(c + 1);
        if let Some(s) = (self.selected)() {
            if s >= c {
                self.selected.set(Some(s + 1));
            }
        }
    }

    /// Turn the keys typed so far (with tone `t`) into a block at the caret.
    fn commit(mut self, t: Tone) {
        if self.typed.read().is_empty() {
            return;
        }
        let s = keyboard::compose(&self.typed.read(), (self.rhotic)(), t);
        let at = self.caret();
        self.insert(Written::Syl(s));
        self.selected.set(Some(at));
        self.typed.set(Vec::new());
        self.rhotic.set(false);
    }

    /// Insert text (or a space / line break) at the caret, first finishing
    /// any half-typed block.
    fn insert_text(self, w: Written) {
        self.commit(Tone::Neutral);
        self.insert(w);
    }

    fn press(mut self, slot: keyboard::Slot) {
        match slot {
            keyboard::Slot::Tone(t) => self.commit(t),
            keyboard::Slot::Key(k) => {
                let next = keyboard::next_keys(&self.typed.read());
                if k == keyboard::ER && !next.contains(&k) {
                    self.rhotic.set(true);
                } else {
                    self.typed.write().push(k);
                }
            }
        }
    }

    /// Undo the last key of the block being typed, or delete before the caret.
    fn backspace(mut self) {
        if (self.rhotic)() {
            self.rhotic.set(false);
        } else if !self.typed.read().is_empty() {
            self.typed.write().pop();
        } else {
            let c = self.caret();
            if c == 0 {
                return;
            }
            self.written.write().remove(c - 1);
            self.cursor.set(c - 1);
            match (self.selected)() {
                Some(s) if s == c - 1 => self.selected.set(None),
                Some(s) if s > c - 1 => self.selected.set(Some(s - 1)),
                _ => {}
            }
        }
    }

    fn move_caret(mut self, delta: isize) {
        let len = self.written.read().len() as isize;
        let c = (self.caret() as isize + delta).clamp(0, len);
        self.cursor.set(c as usize);
    }

    fn clear(mut self) {
        self.written.set(Vec::new());
        self.cursor.set(0);
        self.selected.set(None);
        self.typed.set(Vec::new());
        self.rhotic.set(false);
    }
}

impl Written {
    fn flow(&self) -> parse::Flow {
        match self {
            Written::Syl(_) => parse::Flow::Block,
            Written::Space => parse::Flow::Space { ideographic: false },
            Written::Text(t) if t == "\u{3000}" => parse::Flow::Space { ideographic: true },
            _ => parse::Flow::Other,
        }
    }
}

fn written_text(w: &[Written]) -> String {
    w.iter()
        .map(|w| match w {
            Written::Syl(s) => keyboard::commit_text(s),
            Written::Text(t) => t.clone(),
            Written::Space => " ".to_string(),
            Written::Newline => "\n".to_string(),
        })
        .collect()
}

#[component]
fn KeyboardTab(input: Signal<String>, size: u32, captions: Captions, colorize: bool) -> Element {
    let pad = Pad {
        written: use_signal(Vec::new),
        cursor: use_signal(|| 0),
        selected: use_signal(|| None),
        typed: use_signal(Vec::new),
        rhotic: use_signal(|| false),
    };
    let selected_item = (pad.selected)().and_then(|i| match pad.written.read().get(i) {
        Some(Written::Syl(s)) => Some(SylItem {
            syl: *s,
            source: s.zhuyin(),
            origin: parse::Origin::Zhuyin,
            sandhi_from: None,
            word: None,
        }),
        _ => None,
    });
    rsx! {
        div { class: "workspace",
            PadView { pad, size, captions, colorize }
            Inspector { item: selected_item }
        }
        Keyboard { pad, input }
    }
}

/// The phonetic area of the Keyboard tab. Clicking the left or right half of
/// a block puts the caret before or after it (and shows it in the inspector);
/// clicking empty space puts the caret at the end.
#[component]
fn PadView(pad: Pad, size: u32, captions: Captions, colorize: bool) -> Element {
    let written = pad.written.read().clone();
    let n = written.len();
    let caret = pad.caret();
    let typed = pad.typed.read().clone();
    let composing = (!typed.is_empty()).then(|| keyboard::compose(&typed, (pad.rhotic)(), Tone::Neutral));
    let selected = (pad.selected)();
    let blocks = written.iter().filter(|w| matches!(w, Written::Syl(_))).count();
    let wide = parse::wide_spaces(&written.iter().map(Written::flow).collect::<Vec<_>>());
    let mut cursor = pad.cursor;
    rsx! {
        section { class: if colorize { "panel output-panel pad-panel colorize" } else { "panel output-panel pad-panel" },
            div { class: "output-head",
                span { class: "muted",
                    "{blocks} blocks · click the left or right half of a block to put the caret there"
                }
            }
            div { class: "output pad", onclick: move |_| cursor.set(n),
                for i in 0..=n {
                    if i == caret {
                        if let Some(c) = composing {
                            span { class: "pad-composing",
                                Glyph { syl: c, size }
                            }
                        }
                        span { class: "pad-caret", style: "height: {size * 3 / 4}px;" }
                    }
                    if i < n {
                        PadItem {
                            w: written[i].clone(),
                            index: i,
                            wide: wide[i],
                            size,
                            captions,
                            selected: selected == Some(i),
                            pad,
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn PadItem(
    w: Written,
    index: usize,
    /// for spaces: draw one block wide
    wide: bool,
    size: u32,
    captions: Captions,
    selected: bool,
    pad: Pad,
) -> Element {
    let (mut cursor, mut sel) = (pad.cursor, pad.selected);
    let is_block = matches!(w, Written::Syl(_));
    let mut pick = move |e: MouseEvent, after: bool| {
        e.stop_propagation();
        cursor.set(if after { index + 1 } else { index });
        if is_block {
            sel.set(Some(index));
        }
    };
    let body = match &w {
        Written::Syl(s) => {
            let zhuyin = s.zhuyin();
            let pinyin = s.pinyin();
            let show_zhuyin = matches!(captions, Captions::Zhuyin | Captions::Both);
            let show_pinyin = matches!(captions, Captions::Pinyin | Captions::Both);
            rsx! {
                Glyph { syl: *s, size }
                if show_zhuyin {
                    span { class: "cap zhuyin", "{zhuyin}" }
                }
                if show_pinyin {
                    span { class: "cap", "{pinyin}" }
                }
            }
        }
        Written::Text(t) => {
            let fs = size as f32 * 0.5;
            rsx! {
                span { class: if sits_low(t) { "pad-text low" } else { "pad-text" }, style: "height: {size}px; font-size: {fs}px;", "{t}" }
            }
        }
        Written::Space => {
            let wpx = if wide { size } else { size / 4 };
            rsx! {
                span { class: "pad-space", style: "width: {wpx}px; height: {size}px;" }
            }
        }
        Written::Newline => {
            return rsx! {
                div { class: "break" }
            };
        }
    };
    rsx! {
        span { class: if selected { "pad-item selected" } else { "pad-item" },
            {body}
            button {
                class: "hit left",
                "aria-label": "put the caret before this",
                onclick: move |e| pick(e, false),
            }
            button {
                class: "hit right",
                "aria-label": "put the caret after this",
                onclick: move |e| pick(e, true),
            }
        }
    }
}

/// A touch keyboard for the script. Its keys are the script's own components
/// on a layout adapted from the zhuyin (大千) keyboard. Keys that cannot
/// continue the block being typed are disabled (Fixed) or hidden (Dynamic).
/// A tone key submits the block at the caret.
#[component]
fn Keyboard(pad: Pad, input: Signal<String>) -> Element {
    let mut key_labels = use_signal(|| KeyLabels::Zhuyin);
    let mut board = use_signal(|| Board::Letters);
    let mut shift = use_signal(|| false);
    let mut dynamic = use_signal(|| false);

    let current = pad.typed.read().clone();
    let rhotic = (pad.rhotic)();
    let next = keyboard::next_keys(&current);
    let complete = keyboard::is_complete(&current);
    let can_r = !rhotic && keyboard::can_add_r(&current);
    let has_typed = !current.is_empty();
    let preview = keyboard::compose(&current, rhotic, Tone::Neutral);
    let status = if !has_typed {
        "Type a block: initial, medial, rhyme, then a tone.".to_string()
    } else {
        let mut z: String = current.iter().map(|k| k.zhuyin()).collect();
        if rhotic {
            z.push('ㄦ');
        }
        if complete || rhotic {
            format!("{z} · {}: pick a tone", preview.pinyin_plain())
        } else {
            format!("{z} · keep going, or press a tone to submit it as it is")
        }
    };
    // every key with whether it can continue the block; the Dynamic layout
    // drops the ones that cannot
    let rows: Vec<Vec<(keyboard::Slot, bool)>> = keyboard::layout()
        .into_iter()
        .map(|row| {
            row.into_iter()
                .map(|slot| {
                    let enabled = match slot {
                        keyboard::Slot::Tone(_) => has_typed,
                        keyboard::Slot::Key(k) => next.contains(&k) || (k == keyboard::ER && can_r),
                    };
                    (slot, enabled)
                })
                .filter(|(_, enabled)| *enabled || !dynamic())
                .collect()
        })
        .collect();
    let labels = key_labels();
    let on_press = move |slot: keyboard::Slot| pad.press(slot);
    let pad_empty = pad.written.read().is_empty();

    rsx! {
        section { class: "panel keyboard colorize",
            div { class: "kb-head",
                h3 { "Keyboard" }
                span { class: "kb-status", "{status}" }
                div { class: "kb-tools",
                    div { class: "segmented", role: "group", "aria-label": "Keyboard layout",
                        button {
                            class: if dynamic() { "seg" } else { "seg active" },
                            title: "keys stay in place; impossible ones are disabled",
                            onclick: move |_| dynamic.set(false),
                            "Fixed"
                        }
                        button {
                            class: if dynamic() { "seg active" } else { "seg" },
                            title: "only the keys that can continue the block are shown",
                            onclick: move |_| dynamic.set(true),
                            "Dynamic"
                        }
                    }
                    select {
                        class: "caption-select",
                        "aria-label": "Key labels",
                        onchange: move |e| {
                            key_labels
                                .set(
                                    match e.value().as_str() {
                                        "pinyin" => KeyLabels::Pinyin,
                                        "both" => KeyLabels::Both,
                                        _ => KeyLabels::Zhuyin,
                                    },
                                )
                        },
                        option { value: "zhuyin", selected: labels == KeyLabels::Zhuyin, "Zhuyin labels" }
                        option { value: "pinyin", selected: labels == KeyLabels::Pinyin, "Pinyin labels" }
                        option { value: "both", selected: labels == KeyLabels::Both, "Zhuyin + pinyin" }
                    }
                    button {
                        class: "chip",
                        disabled: pad_empty,
                        title: "append what is written here to the Text tab",
                        onclick: move |_| {
                            let text = written_text(&pad.written.read());
                            input.write().push_str(&text);
                        },
                        "Copy to Text tab"
                    }
                    button {
                        class: "chip",
                        disabled: pad_empty && !has_typed,
                        onclick: move |_| pad.clear(),
                        "Clear"
                    }
                }
            }
            div { class: "kb-board",
                for (r , row) in rows.into_iter().enumerate() {
                    // in Dynamic mode an emptied row collapses (row 4 keeps ABC and ⌫)
                    if !(board() == Board::Letters && row.is_empty() && r != 3) {
                        div { class: "kb-row",
                            if r == 3 {
                                button {
                                    class: if board() == Board::Latin { "kb-key kb-fn kb-toggle active" } else { "kb-key kb-fn kb-toggle" },
                                    title: "English letters",
                                    onclick: move |_| board.set(if board() == Board::Latin { Board::Letters } else { Board::Latin }),
                                    "ABC"
                                }
                            }
                            if r == 2 && board() == Board::Latin {
                                button {
                                    class: if shift() { "kb-key kb-fn kb-toggle active" } else { "kb-key kb-fn kb-toggle" },
                                    title: "capital letter",
                                    onclick: move |_| shift.set(!shift()),
                                    "⇧"
                                }
                            }
                            match board() {
                                Board::Letters => rsx! {
                                    for (slot , enabled) in row {
                                        KeySlot { slot, enabled, labels, on_press }
                                    }
                                },
                                other => {
                                    let chars = match other {
                                        Board::Numbers => keyboard::NUMBERS[r],
                                        Board::Latin => keyboard::LATIN[r],
                                        _ => keyboard::EMOJI[r],
                                    };
                                    let caps = other == Board::Latin && shift();
                                    rsx! {
                                        for t in chars.split_whitespace() {
                                            button {
                                                class: match other {
                                                    Board::Emoji => "kb-key kb-char emoji",
                                                    Board::Latin => "kb-key kb-char latin",
                                                    _ => "kb-key kb-char",
                                                },
                                                onclick: move |_| {
                                                    let text = if caps { t.to_uppercase() } else { t.to_string() };
                                                    pad.insert_text(Written::Text(text));
                                                    shift.set(false);
                                                },
                                                if caps {
                                                    "{t.to_uppercase()}"
                                                } else {
                                                    "{t}"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            if r == 3 {
                                button {
                                    class: "kb-key kb-fn",
                                    title: "undo the last key, or delete before the caret",
                                    onclick: move |_| pad.backspace(),
                                    "⌫"
                                }
                            }
                        }
                    }
                }
                div { class: "kb-row",
                    button {
                        class: if board() == Board::Numbers { "kb-key kb-fn kb-toggle active" } else { "kb-key kb-fn kb-toggle" },
                        title: "numbers and punctuation",
                        onclick: move |_| board.set(if board() == Board::Numbers { Board::Letters } else { Board::Numbers }),
                        "123"
                    }
                    button {
                        class: if board() == Board::Emoji { "kb-key kb-fn kb-toggle active" } else { "kb-key kb-fn kb-toggle" },
                        title: "emoji",
                        onclick: move |_| board.set(if board() == Board::Emoji { Board::Letters } else { Board::Emoji }),
                        "☺"
                    }
                    button {
                        class: "kb-key kb-fn kb-space",
                        title: "a space between words",
                        onclick: move |_| pad.insert_text(Written::Space),
                        "space"
                    }
                    button {
                        class: "kb-key kb-fn",
                        title: "move the caret left",
                        onclick: move |_| pad.move_caret(-1),
                        "◀"
                    }
                    button {
                        class: "kb-key kb-fn",
                        title: "move the caret right",
                        onclick: move |_| pad.move_caret(1),
                        "▶"
                    }
                    button {
                        class: "kb-key kb-fn",
                        title: "new line",
                        onclick: move |_| pad.insert_text(Written::Newline),
                        "⏎"
                    }
                }
            }
        }
    }
}

#[component]
fn KeySlot(
    slot: keyboard::Slot,
    enabled: bool,
    labels: KeyLabels,
    on_press: EventHandler<keyboard::Slot>,
) -> Element {
    let (part, label, class) = match slot {
        keyboard::Slot::Key(k) => {
            let part = match k {
                keyboard::Key::Initial(c) => Part::Consonant(c),
                keyboard::Key::Glide(v) => Part::Vowel(v),
                keyboard::Key::Rhyme(v, c, r) => Part::Rhyme(v, c, r),
            };
            let label = match labels {
                KeyLabels::Zhuyin => k.zhuyin().to_string(),
                KeyLabels::Pinyin => k.pinyin(),
                KeyLabels::Both => format!("{} {}", k.zhuyin(), k.pinyin()),
            };
            (part, label, "kb-key")
        }
        keyboard::Slot::Tone(t) => {
            let mark = match t {
                Tone::First => "ˉ",
                Tone::Second => "ˊ",
                Tone::Third => "ˇ",
                Tone::Fourth => "ˋ",
                Tone::Neutral => "˙",
            };
            let label = match labels {
                KeyLabels::Zhuyin => mark.to_string(),
                KeyLabels::Pinyin => format!("tone {}", t.number()),
                KeyLabels::Both => format!("{mark} {}", t.number()),
            };
            (Part::Tone(t), label, "kb-key kb-tone")
        }
    };
    rsx! {
        button { class, disabled: !enabled, onclick: move |_| on_press.call(slot),
            PartGlyph { part, size: 36 }
            span { class: "kb-zy", "{label}" }
        }
    }
}

// ---------------------------------------------------------------------------
// Glyph rendering
// ---------------------------------------------------------------------------

#[component]
fn Knob(
    label: &'static str,
    hint: &'static str,
    value: f32,
    #[props(default = 0.0)] min: f32,
    #[props(default = 1.0)] max: f32,
    on_change: EventHandler<f32>,
) -> Element {
    let step = (max - min) / 100.0;
    let shown = if max > 1.0 { format!("{value:.2}×") } else { format!("{:.0}", value * 100.0) };
    rsx! {
        label { class: "knob", title: "{hint}",
            div { class: "knob-head",
                span { "{label}" }
                span { class: "muted", "{shown}" }
            }
            input {
                r#type: "range",
                min: "{min}",
                max: "{max}",
                step: "{step}",
                value: "{value}",
                oninput: move |e| on_change.call(e.value().parse().unwrap_or(value)),
            }
            div { class: "knob-hint", "{hint}" }
        }
    }
}

/// A word underline: one 横 stroke drawn with the current brush, at the
/// blocks' stroke weight, inset a third of a block at each end.
#[component]
fn Underline(blocks: usize, size: u32, seed: u32) -> Element {
    let brush = use_context::<Signal<Brush>>();
    let b = brush();
    let s = size as f32;
    // each block cell is the glyph plus 1px padding on either side
    let width_px = blocks as f32 * (s + 2.0) - 2.0 * (s / 3.0);
    // draw in the glyphs' own units (100 per block), so the weight matches
    let k = 100.0 / s;
    let len = width_px * k;
    let h = 16.0;
    let half = glyph::SW / 2.0;
    let stroke = vec![glyph::Seg::M(half, h / 2.0), glyph::Seg::L(len - half, h / 2.0)];
    let mut d = String::new();
    brush::stroke_outline(&stroke, glyph::SW, &b, seed.wrapping_mul(31).wrapping_add(7), &mut d);
    let h_px = h / k;
    // centre the stroke a little below the blocks, clear of their bottom strokes
    let bottom = -h_px / 2.0 - 0.04 * s;
    rsx! {
        svg {
            class: "word-underline",
            view_box: "0 0 {len} {h}",
            width: "{width_px}",
            height: "{h_px}",
            style: "bottom: {bottom}px;",
            path { d: "{d}" }
        }
    }
}

/// How the blocks of a word are linked, between each pair of neighbours.
#[derive(Clone, Copy, PartialEq)]
enum BridgeKind {
    /// a short 横 across the gap
    Bar,
    /// a small 点 on the gap
    Dot,
    /// a short 竖 on the gap
    Line,
}

/// Word bridges: between each pair of neighbouring blocks of one word, a
/// short stroke drawn with the current brush at the tone marks' weight.
/// For a `Bar`, `length` is its total length as a share of a block width,
/// centred on the gap between the two blocks; for a `Line` it is a share of
/// a block height.
/// Bars and dots sit at the height of the tone marks, and lines hang from
/// the top of the blocks; with `middle` all of them are centred on the
/// blocks' vertical centre.
#[component]
fn Bridges(
    blocks: usize,
    size: u32,
    seed: u32,
    kind: BridgeKind,
    #[props(default = 0.35)] length: f32,
    #[props(default)] middle: bool,
) -> Element {
    let brush = use_context::<Signal<Brush>>();
    let b = brush();
    let s = size as f32;
    let cell = s + 2.0;
    // glyph units: 100 per block
    let k = 100.0 / s;
    let width_px = blocks as f32 * cell;
    let height_px = 2.0 + s;
    // each glyph starts 2px down; inside it the tone marks sit at y = 9.5
    // and the centre is y = 50
    let y = 2.0 * k + if middle { 50.0 } else { 9.5 };
    let mut d = String::new();
    for i in 0..blocks.saturating_sub(1) {
        // the gap between block i and block i + 1
        let boundary = (i + 1) as f32 * cell;
        let cx = boundary * k;
        let stroke = match kind {
            BridgeKind::Bar => {
                let half = length / 2.0 * s;
                vec![glyph::Seg::M((boundary - half) * k, y), glyph::Seg::L((boundary + half) * k, y)]
            }
            // short, falling from top-left to bottom-right
            BridgeKind::Dot => vec![glyph::Seg::M(cx - 1.8, y - 1.8), glyph::Seg::L(cx + 1.8, y + 1.8)],
            BridgeKind::Line => {
                let len = length * 100.0;
                let top = if middle { y - len / 2.0 } else { 2.0 * k };
                vec![glyph::Seg::M(cx, top), glyph::Seg::L(cx, top + len)]
            }
        };
        let salt = seed.wrapping_mul(131).wrapping_add(i as u32 * 17 + 3);
        brush::stroke_outline(&stroke, glyph::TONE_SW, &b, salt, &mut d);
    }
    rsx! {
        svg {
            class: "word-bridges",
            view_box: "0 0 {width_px * k} {height_px * k}",
            width: "{width_px}",
            height: "{height_px}",
            path { d: "{d}" }
        }
    }
}

/// Draws laid-out strokes with the current brush (from context).
#[component]
fn Svg(layers: Vec<Layer>, size: u32, #[props(default)] sandhi: bool) -> Element {
    let brush = use_context::<Signal<Brush>>();
    let b = brush();
    let paths: Vec<(String, String)> = layers
        .iter()
        .enumerate()
        .map(|(k, l)| {
            let mut class = l.role.class().to_string();
            if l.role == Role::Tone && sandhi {
                class.push_str(" sandhi");
            }
            (class, brush::layer_outline(l, &b, k as u32 * 131 + 17))
        })
        .collect();
    rsx! {
        svg {
            class: "glyph",
            width: "{size}",
            height: "{size}",
            view_box: "0 0 100 100",
            for (class , d) in paths {
                path { class: "{class}", d: "{d}" }
            }
        }
    }
}

#[component]
fn Glyph(syl: Syllable, size: u32, #[props(default)] sandhi: bool) -> Element {
    let layers = glyph::syllable_layers(&syl);
    rsx! {
        Svg { layers, size, sandhi }
    }
}

#[component]
fn PartGlyph(part: Part, size: u32) -> Element {
    let layers = glyph::part_layers(part);
    rsx! {
        Svg { layers, size }
    }
}

#[component]
fn ItemView(
    item: Item,
    index: usize,
    /// for spaces: draw one block wide
    wide: bool,
    size: u32,
    captions: Captions,
    selected: bool,
    on_select: EventHandler<usize>,
) -> Element {
    match item {
        Item::Syl(s) => {
            let tip = format!("{}  ·  {}  ·  {}", s.source, s.syl.pinyin(), s.syl.zhuyin());
            let pinyin = s.syl.pinyin();
            let zhuyin = s.syl.zhuyin();
            let show_zhuyin = matches!(captions, Captions::Zhuyin | Captions::Both);
            let show_pinyin = matches!(captions, Captions::Pinyin | Captions::Both);
            rsx! {
                button {
                    class: if selected { "cell selected" } else { "cell" },
                    title: "{tip}",
                    onclick: move |_| on_select.call(index),
                    Glyph { syl: s.syl, size, sandhi: s.sandhi_from.is_some() }
                    if show_zhuyin {
                        span { class: "cap zhuyin", "{zhuyin}" }
                    }
                    if show_pinyin {
                        span { class: "cap", "{pinyin}" }
                    }
                }
            }
        }
        Item::Text(t) => {
            let fs = size as f32 * 0.5;
            rsx! {
                span { class: if sits_low(&t) { "text low" } else { "text" }, style: "height: {size}px; font-size: {fs}px;", "{t}" }
            }
        }
        Item::Space { .. } => {
            // a block plus the cell's 1px side padding, or a narrow gap
            let w = if wide { size + 2 } else { size / 4 };
            rsx! {
                span { class: "space", style: "width: {w}px;" }
            }
        }
        Item::Newline => rsx! {
            div { class: "break" }
        },
    }
}

#[component]
fn Inspector(item: Option<SylItem>) -> Element {
    let Some(s) = item else {
        return rsx! {
            aside { class: "panel inspector empty",
                h3 { "Inspector" }
                p { class: "muted", "Click any block to take it apart." }
                div { class: "colorize demo",
                    Glyph { syl: syl("zhuāng"), size: 132 }
                }
            }
        };
    };
    let parts = glyph::describe(&s.syl);
    let sandhi_note = s
        .sandhi_from
        .map(|t| format!("written tone {} → spoken tone {}", t.number(), s.syl.tone.number()));
    // stroke counts: the phonetic block against the character it stands for
    let (block_strokes, tone_strokes) = glyph::stroke_count(&glyph::syllable_layers(&s.syl));
    let tone_note = if tone_strokes > 0 { " + 1 tone mark" } else { "" };
    let source_strokes: Vec<(char, u8)> = s
        .source
        .chars()
        .filter(|c| parse::is_hanzi(*c))
        .filter_map(|c| hanzi::strokes(c).map(|n| (c, n)))
        .collect();
    // characters that are read exactly like this (citation tone)
    let citation = s.syl.with_tone(s.sandhi_from.unwrap_or(s.syl.tone));
    rsx! {
        aside { class: "panel inspector",
            h3 { "Inspector" }
            div { class: "insp-head",
                div { class: "colorize",
                    Glyph { syl: s.syl, size: 132, sandhi: s.sandhi_from.is_some() }
                }
                div { class: "insp-meta",
                    div { class: "insp-source", "{s.source}" }
                    div { class: "insp-pinyin", "{s.syl.pinyin()}" }
                    div { class: "insp-zhuyin", "{s.syl.zhuyin()}" }
                    if let Some(note) = sandhi_note {
                        div { class: "insp-sandhi", "{note}" }
                    }
                }
            }
            div { class: "insp-strokes",
                div { class: "stroke-stat",
                    span { class: "stat-num", "{block_strokes}" }
                    span { class: "stat-label", "strokes in the phonetic block{tone_note}" }
                }
                for (c , n) in source_strokes {
                    div { class: "stroke-stat",
                        span { class: "stat-num", "{n}" }
                        span { class: "stat-label", "strokes in {c}" }
                    }
                }
            }
            ul { class: "parts",
                for p in parts {
                    li {
                        span { class: "swatch {p.role.class()}" }
                        div {
                            div {
                                b { "{p.role.label()} " }
                                span { class: "sound", "{p.sound}" }
                                span { class: "comp", "{p.component}" }
                            }
                            div { class: "note", "{p.note}" }
                        }
                    }
                }
            }
            Homophones { syl: citation, current: s.source.chars().next() }
        }
    }
}

/// Chinese characters with exactly this syllable and tone.
#[component]
fn Homophones(syl: Syllable, current: Option<char>) -> Element {
    let mut show_all = use_signal(|| false);
    let all = hanzi::homophones(&syl);
    let total = all.len();
    let common = all.iter().filter(|m| m.freq > 0).count();
    let limit = if show_all() { total } else { total.min(36) };
    let shown: Vec<hanzi::Match> = all.into_iter().take(limit).collect();
    let reading = syl.pinyin();
    let zhuyin = syl.zhuyin();
    rsx! {
        div { class: "homophones",
            h4 {
                "Characters read "
                b { "{zhuyin} {reading}" }
                span { class: "muted", " · {total} ({common} common)" }
            }
            if total == 0 {
                p { class: "muted small", "No standard character has exactly this reading." }
            }
            div { class: "hz-list",
                for m in shown {
                    span {
                        class: if Some(m.ch) == current { "hz current" } else if m.freq > 0 { "hz" } else { "hz rare" },
                        title: if m.freq > 0 { format!("{} strokes · {} occurrences in the 现代汉语频率词典 corpus", m.strokes, m.freq) } else { format!("{} strokes · not in the frequency corpus", m.strokes) },
                        span { class: "hz-char", "{m.ch}" }
                        span { class: "hz-strokes", "{m.strokes}" }
                    }
                }
            }
            if total > limit {
                button {
                    class: "chip",
                    onclick: move |_| show_all.set(true),
                    "Show all {total}"
                }
            }
            p { class: "muted small",
                "Most frequent first; the small number is the stroke count. Faded characters are rarer readings outside the frequency corpus."
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Reference key
// ---------------------------------------------------------------------------

#[component]
fn Labeled(syl: Syllable, label: String, #[props(default = 64)] size: u32) -> Element {
    rsx! {
        figure { class: "labeled",
            Glyph { syl, size }
            figcaption { "{label}" }
        }
    }
}

#[component]
fn Reference() -> Element {
    rsx! {
        section { class: "panel reference colorize",
            h2 { "The script" }
            p { class: "lede",
                "Every syllable is one square block, like a Chinese character. It is cut up the way zhuyin cuts it: an initial consonant, a medial glide (ㄧ ㄨ ㄩ) and one rhyme, plus the tone. Each of those is a single symbol, and each symbol is one sound. Nothing about a block depends on its neighbours."
            }

            h3 { "1 · Where the parts go" }
            div { class: "anatomy",
                SlotDiagram {}
                div { class: "anatomy-examples",
                    Labeled { syl: syl("zhuāng"), label: "ㄓㄨㄤ zhuāng".to_string(), size: 96 }
                    Labeled { syl: syl("liù"), label: "ㄌㄧㄡ liù".to_string(), size: 96 }
                    Labeled { syl: syl("guì"), label: "ㄍㄨㄟ guì".to_string(), size: 96 }
                    Labeled { syl: syl("yǒu"), label: "ㄧㄡ yǒu".to_string(), size: 96 }
                    Labeled { syl: syl("shì"), label: "ㄕ shì".to_string(), size: 96 }
                }
            }
            ol { class: "rules",
                li { b { "Tone" } " sits top-centre, above everything, using pinyin's four marks drawn as brush strokes. The neutral tone has no mark." }
                li { b { "Initial" } " (consonant) stands on the left like a radical, the left–right structure of 好 or 吃." }
                li { b { "Medial" } " ㄧ ㄨ ㄩ is flattened on top of the rhyme, the way 艹 sits on top of a character." }
                li { b { "Rhyme" } " is one symbol: its vowel plus at most one ending stroke. Rhymes with the same vowel sound share that vowel (ㄚ → ㄞ ㄠ ㄢ ㄤ); different vowel sounds never share a shape, so ㄜ (me), ㄟ (mei) and ㄝ (ye) look unrelated." }
                li { "Nothing is hidden or dropped: liu is ㄌㄧㄡ, gui is ㄍㄨㄟ, dun is ㄉㄨㄣ, ju is ㄐㄩ, and in, ing, ong are ㄧㄣ, ㄧㄥ, ㄨㄥ, all exactly as in zhuyin." }
                li { "With no initial (ㄚ, ㄧ, ㄨㄛ …) the final takes the full width. With the buzzing vowel of zi ci si zhi chi shi ri, the initial fills the block on its own." }
                li { "Tone sandhi is the only change between neighbouring syllables, and like a dictionary the script writes the citation tone. Turn on " i { "Tone sandhi" } " to see the spoken tones instead." }
                li { "Every symbol is one to three strokes, using only the classical strokes 横 竖 撇 点 捺 提 and their 折 / 钩 / 弯 compounds, at one constant weight." }
            }

            h3 { "2 · Initials: frame = place, inner mark = manner" }
            p { class: "muted",
                "Picture a mouth facing left: lips on the left, the roof of the mouth on top. The frame is drawn along the surface where the closure is made; the single mark inside shows how the air gets out. Every consonant of a place shares one frame, and every consonant of a manner shares one mark."
            }
            InitialChart {}

            h3 { "3 · Rhymes: thirteen symbols, one per sound" }
            p { class: "muted",
                "Zhuyin's thirteen rhymes, each written as one symbol. The four vowel sounds each get their own shape (rows). A rhyme ending in a glide or a nasal adds one stroke to its vowel (columns): a rising foot for -i (ai, ei), a vertical through 八 for ao and the 捺 that completes 又 for ou, a bar under 八 or through 了 for -n, the same bar closed at the back for -ng, and a slash through the top of 了 for er. 儿化 adds the curl 乚 under any rhyme: huār is ㄏㄨㄚㄦ, the ㄚ symbol with the curl."
            }
            RhymeChart {}

            h3 { "4 · Medials ㄧ ㄨ ㄩ" }
            p { class: "muted",
                "The three glides are written full size when they are the whole final (ㄧ yi, ㄨ wu, ㄩ yu) and flattened on top when a rhyme follows (ㄧㄚ, ㄨㄛ, ㄩㄝ)."
            }
            div { class: "row",
                for v in [Vowel::I, Vowel::U, Vowel::Yu] {
                    figure { class: "vowel-card",
                        div { class: "vowel-forms",
                            PartGlyph { part: Part::Vowel(v), size: 72 }
                            div { class: "glide",
                                PartGlyph { part: Part::Glide(v), size: 48 }
                                span { class: "muted", "medial" }
                            }
                        }
                        figcaption {
                            b { "{v.zhuyin()} {v.pinyin()}" }
                            span { class: "comp", " {v.component()}" }
                            div { class: "muted small", "{v.idea()}" }
                        }
                    }
                }
            }

            h3 { "5 · Tones" }
            div { class: "row",
                for t in Tone::ALL {
                    figure { class: "tone-card",
                        div { class: "tone-pair",
                            PartGlyph { part: Part::Tone(t), size: 56 }
                            Glyph { syl: syl("ma").with_tone(t), size: 72 }
                        }
                        figcaption {
                            b { "{tone_title(t)} " }
                            span { class: "muted", "{t.pinyin_mark()} · {t.stroke_name()}" }
                        }
                    }
                }
            }

            h3 { "6 · Every final" }
            p { class: "muted", "In zhuyin order. Each final on its own (left) and after a consonant (right)." }
            FinalsChart {}
        }
    }
}

fn tone_title(t: Tone) -> &'static str {
    match t {
        Tone::First => "1st",
        Tone::Second => "2nd",
        Tone::Third => "3rd",
        Tone::Fourth => "4th",
        Tone::Neutral => "neutral",
    }
}

#[component]
fn SlotDiagram() -> Element {
    // A schematic of the block with the four slots outlined.
    let boxes: [(f32, f32, f32, f32, &str, &str); 4] = [
        (34.0, 2.0, 32.0, 15.0, "tone", "r-tone"),
        (4.0, 21.0, 33.0, 76.0, "initial", "r-initial"),
        (43.0, 21.0, 53.0, 22.0, "medial", "r-medial"),
        (43.0, 49.0, 53.0, 48.0, "rhyme", "r-rhyme"),
    ];
    rsx! {
        svg { class: "slots", width: "220", height: "220", view_box: "-2 -2 104 104",
            rect {
                x: "0",
                y: "0",
                width: "100",
                height: "100",
                class: "slot-frame",
            }
            for (x , y , w , h , label , class) in boxes {
                g { class: "{class}",
                    rect {
                        x: "{x}",
                        y: "{y}",
                        width: "{w}",
                        height: "{h}",
                        rx: "2",
                        class: "slot",
                    }
                    text {
                        x: "{x + w / 2.0}",
                        y: "{y + h / 2.0 + 2.2}",
                        text_anchor: "middle",
                        class: "slot-label",
                        "{label}"
                    }
                }
            }
        }
    }
}

#[component]
fn InitialChart() -> Element {
    rsx! {
        div { class: "table-wrap",
            table { class: "chart",
                thead {
                    tr {
                        th {}
                        for m in Manner::ALL {
                            th {
                                div { class: "th-title", "{m.name()}" }
                                div { class: "th-mark", "{m.mark()}" }
                                div { class: "th-idea", "{m.idea()}" }
                            }
                        }
                    }
                }
                tbody {
                    for p in Place::ALL {
                        tr {
                            th { class: "row-head",
                                div { class: "th-mark big", "{p.frame()}" }
                                div { class: "th-title", "{p.name()}" }
                                div { class: "th-idea", "{p.idea()}" }
                            }
                            for m in Manner::ALL {
                                InitialCell { c: Consonant::new(p, m) }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn InitialCell(c: Consonant) -> Element {
    if !c.exists() {
        return rsx! {
            td { class: "empty" }
        };
    }
    let syllabic_only = c == Consonant::NG;
    rsx! {
        td {
            PartGlyph { part: Part::Consonant(c), size: 58 }
            div { class: "cell-label",
                b { "{c.zhuyin()} " }
                span { class: "muted", "{c.pinyin()}" }
            }
            if syllabic_only {
                div { class: "th-idea", "only in 嗯 ńg" }
            }
        }
    }
}

/// Ending columns of the rhyme chart; `None` inside is the r curl.
const ENDINGS: [Option<Option<Coda>>; 6] = [
    None,
    Some(Some(Coda::I)),
    Some(Some(Coda::U)),
    Some(Some(Coda::N)),
    Some(Some(Coda::Ng)),
    Some(None),
];

fn rhyme_of(v: Vowel, e: Option<Option<Coda>>) -> Syllable {
    Syllable {
        nucleus: Some(v),
        coda: e.flatten(),
        rhotic: e == Some(None),
        ..Syllable::empty()
    }
}

#[component]
fn RhymeChart() -> Element {
    rsx! {
        div { class: "table-wrap",
            table { class: "chart rhymes",
                thead {
                    tr {
                        th {}
                        for e in ENDINGS {
                            th {
                                match e {
                                    None => rsx! {
                                        div { class: "th-title", "Vowel alone" }
                                        div { class: "th-idea", "no ending" }
                                    },
                                    Some(c) => {
                                        let (name, stroke, idea) = match c {
                                            Some(c) => (format!("-{}", c.pinyin()), c.stroke(), c.idea()),
                                            None => ("-r".to_string(), R_STROKE, R_IDEA),
                                        };
                                        rsx! {
                                            div { class: "end-glyph",
                                                PartGlyph { part: Part::Ending(c), size: 40 }
                                            }
                                            div { class: "th-title", "{name}" }
                                            div { class: "th-mark small-mark", "{stroke}" }
                                            div { class: "th-idea", "{idea}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                tbody {
                    for v in [Vowel::A, Vowel::O, Vowel::E, Vowel::Ehat] {
                        tr {
                            th { class: "row-head",
                                div { class: "th-mark big", "{v.component()}" }
                                div { class: "th-title", "{v.zhuyin()} {v.pinyin()}" }
                                div { class: "th-idea", "{v.idea()}" }
                            }
                            for e in ENDINGS {
                                RhymeCell { s: rhyme_of(v, e) }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn RhymeCell(s: Syllable) -> Element {
    let zhuyin = s.rhyme_zhuyin();
    // the r column holds only ㄦ; 儿化 on other rhymes is explained in the text
    if zhuyin.is_empty() || (s.rhotic && s.nucleus != Some(Vowel::E)) {
        return rsx! {
            td { class: "empty" }
        };
    }
    let Some(v) = s.nucleus else {
        return rsx! {
            td { class: "empty" }
        };
    };
    let name = s.rhyme_name();
    rsx! {
        td {
            PartGlyph { part: Part::Rhyme(v, s.coda, s.rhotic), size: 64 }
            div { class: "cell-label",
                b { "{zhuyin} " }
                span { class: "muted", "{name}" }
            }
        }
    }
}

/// (zhuyin, pinyin, zero-initial example, example with a consonant)
const FINAL_EXAMPLES: &[(&str, &str, &str, &str)] = &[
    ("ㄚ", "a", "a", "ba"),
    ("ㄛ", "o", "o", "bo"),
    ("ㄜ", "e", "e", "de"),
    ("ㄝ", "ê", "ê", ""),
    ("ㄞ", "ai", "ai", "dai"),
    ("ㄟ", "ei", "ei", "bei"),
    ("ㄠ", "ao", "ao", "dao"),
    ("ㄡ", "ou", "ou", "dou"),
    ("ㄢ", "an", "an", "dan"),
    ("ㄣ", "en", "en", "ben"),
    ("ㄤ", "ang", "ang", "dang"),
    ("ㄥ", "eng", "", "deng"),
    ("ㄦ", "er", "er", ""),
    ("(none)", "-i", "", "zhi"),
    ("ㄧ", "i", "yi", "di"),
    ("ㄧㄚ", "ia", "ya", "jia"),
    ("ㄧㄝ", "ie", "ye", "die"),
    ("ㄧㄠ", "iao", "yao", "diao"),
    ("ㄧㄡ", "iu", "you", "diu"),
    ("ㄧㄢ", "ian", "yan", "dian"),
    ("ㄧㄣ", "in", "yin", "bin"),
    ("ㄧㄤ", "iang", "yang", "jiang"),
    ("ㄧㄥ", "ing", "ying", "ding"),
    ("ㄨ", "u", "wu", "du"),
    ("ㄨㄚ", "ua", "wa", "gua"),
    ("ㄨㄛ", "uo", "wo", "duo"),
    ("ㄨㄞ", "uai", "wai", "guai"),
    ("ㄨㄟ", "ui", "wei", "dui"),
    ("ㄨㄢ", "uan", "wan", "duan"),
    ("ㄨㄣ", "un", "wen", "dun"),
    ("ㄨㄤ", "uang", "wang", "guang"),
    ("ㄨㄥ", "ong / weng", "weng", "dong"),
    ("ㄩ", "ü", "yu", "lü"),
    ("ㄩㄝ", "üe", "yue", "jue"),
    ("ㄩㄢ", "üan", "yuan", "juan"),
    ("ㄩㄣ", "ün", "yun", "jun"),
    ("ㄩㄥ", "iong", "yong", "jiong"),
];

#[component]
fn FinalsChart() -> Element {
    rsx! {
        div { class: "finals",
            for (zhuyin , pinyin , bare , with) in FINAL_EXAMPLES.iter() {
                div { class: "final-card",
                    div { class: "final-name",
                        span { class: "zy", "{zhuyin}" }
                        span { class: "muted", " {pinyin}" }
                    }
                    div { class: "final-glyphs",
                        if !bare.is_empty() {
                            Labeled { syl: syl(bare), label: bare.to_string(), size: 56 }
                        }
                        if !with.is_empty() {
                            Labeled { syl: syl(with), label: with.to_string(), size: 56 }
                        }
                    }
                }
            }
        }
    }
}
