mod glyph;
mod parse;
mod phonology;

use dioxus::prelude::*;
use glyph::{Layer, Part, Role};
use parse::{Item, SylItem};
use phonology::{Coda, Consonant, Manner, Place, Syllable, Tone, Vowel};

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
    let mut captions = use_signal(|| true);
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
                        label { class: "toggle",
                            input {
                                r#type: "checkbox",
                                checked: captions(),
                                onchange: move |e| captions.set(e.checked()),
                            }
                            "Pinyin captions"
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
    captions: bool,
    selected: bool,
    on_select: EventHandler<usize>,
) -> Element {
    match item {
        Item::Syl(s) => {
            let tip = format!("{}  ·  {}  ·  {}", s.source, s.syl.pinyin(), s.syl.zhuyin());
            let caption = s.syl.pinyin();
            rsx! {
                button {
                    class: if selected { "cell selected" } else { "cell" },
                    title: "{tip}",
                    onclick: move |_| on_select.call(index),
                    Glyph { syl: s.syl, size, sandhi: s.sandhi_from.is_some() }
                    if captions {
                        span { class: "cap", "{caption}" }
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
                "Every syllable is one square block, like a Chinese character. A block is assembled from at most six parts that always sit in the same places, and every part is only one or two strokes, so a reader who knows six frames, four marks and seven vowels can read any Mandarin syllable. Nothing about a block depends on its neighbours."
            }

            h3 { "1 · Where the parts go" }
            div { class: "anatomy",
                SlotDiagram {}
                div { class: "anatomy-examples",
                    Labeled { syl: syl("zhuāng"), label: "zhuāng: all slots".to_string(), size: 96 }
                    Labeled { syl: syl("xué"), label: "xué".to_string(), size: 96 }
                    Labeled { syl: syl("yǒu"), label: "yǒu: no initial".to_string(), size: 96 }
                    Labeled { syl: syl("shì"), label: "shì: initial only".to_string(), size: 96 }
                    Labeled { syl: syl("huār"), label: "huār: with 儿化".to_string(), size: 96 }
                }
            }
            ol { class: "rules",
                li { b { "Tone" } " sits top-centre, above everything, using pinyin's own four marks drawn as brush strokes. The neutral tone has no mark." }
                li { b { "Initial" } " (consonant) stands on the left like a radical, the left–right structure of 好 or 吃." }
                li { b { "Final" } " stacks on the right from top to bottom, like the right half of 漫: " i { "medial" } " glide, main " i { "vowel" } ", " i { "coda" } ". Glides and codas are flattened, like 艹 on top or 灬 below, so the vowel stays the visual centre." }
                li { "With no initial (a, yi, wu, yu …) the final takes the full width. With an empty rhyme (zi ci si zhi chi shi ri) the initial fills the block on its own, as in zhuyin." }
                li { "Spelling is phonemic, one sound per component: " code { "iu" } " is i·o·u, " code { "ui" } " is u·e·i, " code { "un" } " is u·e·n, " code { "ju" } " is j·ü, " code { "ie" } " is i·ê, " code { "ao" } " is a·u, " code { "er" } " is e + r. Pinyin's y/w spellings disappear." }
                li { "Tone sandhi is the only change between neighbouring syllables, and like a dictionary the script writes the citation tone. Turn on " i { "Tone sandhi" } " to see the spoken tones instead." }
                li { "Every part is one or two strokes, and only the classical strokes are used: 横 竖 撇 点 捺 提 and their 折 / 钩 / 弯 compounds, at one constant weight." }
            }

            h3 { "2 · Initials: frame = place, inner mark = manner" }
            p { class: "muted",
                "Picture a mouth facing left: lips on the left, the roof of the mouth on top. The frame is drawn along the surface where the closure is made; the single mark inside shows how the air gets out. Every consonant of a place shares one frame, and every consonant of a manner shares one mark."
            }
            InitialChart {}

            h3 { "3 · Vowels" }
            p { class: "muted",
                "Vowels are free-standing strokes, consonants are frames, so the two never look alike. A dot fronts a vowel: e → ê, u → ü. i, u and ü have a flattened form for when they glide: on top as a medial, at the bottom as a coda."
            }
            VowelChart {}

            h3 { "4 · Codas" }
            p { class: "muted",
                "The syllable's ending reuses what you already know. -i and -u are the flattened vowels. After a vowel only n, ng or r can follow, so those are written as the bare frame of their place with no mark: 厂 front for -n, 𠃌 back for -ng, and the curl 乚 for -r. The frame alone shows the n/ng difference."
            }
            div { class: "row",
                PartCard { part: Part::Glide(Vowel::I), title: "-i", sub: "乙 flat · ai ei uai uei" }
                PartCard { part: Part::Glide(Vowel::U), title: "-u", sub: "乂 flat · ao ou iao iou" }
                PartCard { part: Part::Coda(Coda::N), title: "-n", sub: "bare 厂 · an en in un" }
                PartCard { part: Part::Coda(Coda::Ng), title: "-ng", sub: "bare 𠃌 · ang eng ing ong" }
                PartCard { part: Part::Rhotic, title: "-r", sub: "bare 乚 · er and 儿化" }
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
            p { class: "muted", "Each final on its own (left) and after a consonant (right)." }
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
fn PartCard(part: Part, title: String, sub: String) -> Element {
    rsx! {
        figure { class: "part-card",
            PartGlyph { part, size: 72 }
            figcaption {
                b { "{title}" }
                span { class: "muted", "{sub}" }
            }
        }
    }
}

#[component]
fn SlotDiagram() -> Element {
    // A schematic of the block with the six slots outlined.
    let boxes: [(f32, f32, f32, f32, &str, &str); 6] = [
        (34.0, 2.0, 32.0, 15.0, "tone", "r-tone"),
        (4.0, 21.0, 33.0, 76.0, "initial", "r-initial"),
        (43.0, 21.0, 53.0, 20.0, "medial", "r-medial"),
        (43.0, 46.0, 53.0, 26.0, "vowel", "r-nucleus"),
        (43.0, 77.0, 53.0, 20.0, "coda", "r-coda"),
        (43.0, 77.0, 0.0, 0.0, "", ""),
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
            for (x , y , w , h , label , class) in boxes.into_iter().filter(|b| b.2 > 0.0) {
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
    let coda_only = c == Consonant::NG;
    rsx! {
        td {
            PartGlyph { part: Part::Consonant(c), size: 58 }
            div { class: "cell-label",
                b { "{c.pinyin()}" }
                span { class: "muted", " {c.zhuyin()}" }
            }
            if coda_only {
                div { class: "th-idea", "only in 嗯 ńg; as a coda, the bare frame" }
            }
        }
    }
}

#[component]
fn VowelChart() -> Element {
    rsx! {
        div { class: "row",
            for v in Vowel::ALL {
                figure { class: "vowel-card",
                    div { class: "vowel-forms",
                        PartGlyph { part: Part::Vowel(v), size: 72 }
                        if matches!(v, Vowel::I | Vowel::U | Vowel::Yu) {
                            div { class: "glide",
                                PartGlyph { part: Part::Glide(v), size: 48 }
                                span { class: "muted", "glide" }
                            }
                        }
                    }
                    figcaption {
                        b { "{v.pinyin()}" }
                        span { class: "comp", " {v.component()}" }
                        div { class: "muted small", "{v.idea()}" }
                    }
                }
            }
        }
    }
}

/// (final, zero-initial example, example with a consonant)
const FINAL_EXAMPLES: &[(&str, &str, &str)] = &[
    ("a", "a", "ba"),
    ("o", "o", "bo"),
    ("e", "e", "de"),
    ("ê", "ê", ""),
    ("ai", "ai", "dai"),
    ("ei", "ei", "bei"),
    ("ao", "ao", "dao"),
    ("ou", "ou", "dou"),
    ("an", "an", "dan"),
    ("en", "en", "ben"),
    ("ang", "ang", "dang"),
    ("eng", "eng", "deng"),
    ("ong", "", "dong"),
    ("er", "er", ""),
    ("-i", "", "zhi"),
    ("i", "yi", "di"),
    ("ia", "ya", "jia"),
    ("ie", "ye", "die"),
    ("iao", "yao", "diao"),
    ("iu (iou)", "you", "diu"),
    ("ian", "yan", "dian"),
    ("in", "yin", "bin"),
    ("iang", "yang", "jiang"),
    ("ing", "ying", "ding"),
    ("iong", "yong", "jiong"),
    ("u", "wu", "du"),
    ("ua", "wa", "gua"),
    ("uo", "wo", "duo"),
    ("uai", "wai", "guai"),
    ("ui (uei)", "wei", "dui"),
    ("uan", "wan", "duan"),
    ("un (uen)", "wen", "dun"),
    ("uang", "wang", "guang"),
    ("ueng", "weng", ""),
    ("ü", "yu", "lü"),
    ("üe", "yue", "jue"),
    ("üan", "yuan", "juan"),
    ("ün", "yun", "jun"),
];

#[component]
fn FinalsChart() -> Element {
    rsx! {
        div { class: "finals",
            for (name , bare , with) in FINAL_EXAMPLES.iter() {
                div { class: "final-card",
                    div { class: "final-name", "{name}" }
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
