mod glyph;
mod parse;
mod phonology;

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

fn syl(p: &str) -> Syllable {
    parse::parse_pinyin_token(p)
        .and_then(|v| v.into_iter().next())
        .map(|(s, _)| s)
        .unwrap_or_else(Syllable::empty)
}

#[component]
fn App() -> Element {
    let mut input = use_signal(|| EXAMPLES[0].1.to_string());
    let mut size = use_signal(|| 84u32);
    let mut captions = use_signal(|| Captions::Zhuyin);
    let mut colorize = use_signal(|| false);
    let mut sandhi = use_signal(|| false);
    let mut selected = use_signal(|| None::<usize>);

    let items = use_memo(move || {
        let mut items = parse::convert(&input.read());
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

    rsx! {
        document::Title { "音块 Yīnkuài" }
        document::Stylesheet { href: CSS }
        div { class: "page",
            header { class: "masthead",
                div { class: "logo colorize",
                    Glyph { syl: syl("yīn"), size: 54 }
                    Glyph { syl: syl("kuài"), size: 54 }
                }
                div { class: "title",
                    h1 { "音块 " span { class: "latin", "Yīnkuài" } }
                    p { "Sound-blocks: a featural, one-block-per-syllable script for Mandarin, built from the strokes and components of Chinese writing." }
                }
            }

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
                div { class: "toolbar",
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
                            title: "Show spoken tones (3-3 → 2-3, 一 and 不 changes). Changed marks turn orange.",
                            input {
                                r#type: "checkbox",
                                checked: sandhi(),
                                onchange: move |e| sandhi.set(e.checked()),
                            }
                            "Tone sandhi"
                        }
                        label { class: "toggle size",
                            "Size"
                            input {
                                r#type: "range",
                                min: "40",
                                max: "160",
                                value: "{size}",
                                oninput: move |e| size.set(e.value().parse().unwrap_or(84)),
                            }
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
                        for (i , item) in items.read().iter().enumerate() {
                            ItemView {
                                key: "{i}",
                                item: item.clone(),
                                index: i,
                                size: size(),
                                captions: captions(),
                                selected: selected() == Some(i),
                                on_select: move |i| selected.set(Some(i)),
                            }
                        }
                    }
                }
                Inspector { item: selected_item }
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

// ---------------------------------------------------------------------------
// Glyph rendering
// ---------------------------------------------------------------------------

#[component]
fn Svg(layers: Vec<Layer>, size: u32, #[props(default)] sandhi: bool) -> Element {
    let paths: Vec<(String, String, f32)> = layers
        .iter()
        .map(|l| {
            let mut class = l.role.class().to_string();
            if l.role == Role::Tone && sandhi {
                class.push_str(" sandhi");
            }
            (class, l.d.clone(), l.width)
        })
        .collect();
    rsx! {
        svg {
            class: "glyph",
            width: "{size}",
            height: "{size}",
            view_box: "0 0 100 100",
            for (class , d , w) in paths {
                path { class: "{class}", d: "{d}", stroke_width: "{w}" }
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
                span { class: "text", style: "height: {size}px; font-size: {fs}px;", "{t}" }
            }
        }
        Item::Space => {
            let w = size / 4;
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
                "Zhuyin's thirteen rhymes, each written as one symbol. The four vowel sounds each get their own shape (rows). A rhyme ending in a glide or a nasal adds a single stroke to its vowel (columns), the same stroke for every vowel: a rising 提 for -i, a bowl for -u, a bar for -n, the bar closed at the back for -ng, and the retroflex curl for -r. 儿化 adds that same curl to any rhyme: huār is ㄏㄨㄚㄦ, the ㄚ symbol with the curl."
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
