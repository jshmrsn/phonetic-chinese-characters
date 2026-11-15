//! Writes a static HTML contact sheet of glyphs for visual review.
//! cargo run --example sheet --no-default-features -- out.html "ba bo ..."
#[path = "../src/phonology.rs"]
#[allow(dead_code)]
mod phonology;
#[path = "../src/parse.rs"]
#[allow(dead_code)]
mod parse;
#[path = "../src/glyph.rs"]
#[allow(dead_code)]
mod glyph;

fn main() {
    let mut args = std::env::args().skip(1);
    let out = args.next().unwrap_or_else(|| "sheet.html".into());
    let text = args.next().unwrap_or_default();
    let size: u32 = args.next().and_then(|s| s.parse().ok()).unwrap_or(120);
    let mut html = String::from("<!doctype html><meta charset=utf-8><style>body{background:#f5f0e6;color:#1f1b16;font:13px sans-serif;margin:16px}.g{display:inline-flex;flex-direction:column;align-items:center;margin:4px 6px}svg{background:#fffdf8;outline:1px solid #e2d9c8}path{fill:none;stroke:currentColor;stroke-linecap:round;stroke-linejoin:round}.r-tone{stroke:#7a3fa8}.r-initial{stroke:#b23a2b}.r-medial{stroke:#1f8a7a}.r-nucleus{stroke:#2c4ea3}.r-coda{stroke:#b0700f}.r-rhotic{stroke:#b0407a}.mono path{stroke:#1f1b16}</style>");
    for line in text.split('|') {
        html.push_str("<div>");
        for item in parse::convert(line) {
            if let parse::Item::Syl(s) = item {
                html.push_str(&format!("<div class=g><svg width={size} height={size} viewBox='0 0 100 100'>"));
                for l in glyph::syllable_layers(&s.syl) {
                    html.push_str(&format!("<path class='{}' d='{}' stroke-width='{}'/>", l.role.class(), l.d, l.width));
                }
                html.push_str(&format!("</svg>{}</div>", s.syl.pinyin()));
            }
        }
        html.push_str("</div>");
    }
    std::fs::write(&out, html).unwrap();
}
