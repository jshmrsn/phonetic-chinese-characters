# 音块 Yīnkuài

A featural, one-block-per-syllable phonetic script for Mandarin, with a Rust +
Dioxus app that converts Hanzi, pinyin (tone marks or numbers) or zhuyin into
the script. It has two tabs, **Text** (type or paste, see it rendered) and
**Keyboard** (write blocks directly into an editable phonetic area). Both share
the display settings and have an inspector. The app also has:

* a brush simulator that renders the strokes anywhere from plain geometric
  lines to handwriting, with tuning knobs;
* an inspector that takes a block apart, counts its strokes, and lists the
  Chinese characters with the same reading and their stroke counts;
* an on-screen keyboard on the zhuyin (大千) layout that offers only the keys
  that can still make a real syllable. A Fixed layout disables the rest and a
  Dynamic layout hides them. A writing pad collects the result.

See [DESIGN.md](DESIGN.md) for the full ruleset. The app also contains an
illustrated reference.

## Run

Web (default):

```bash
dx serve --web
```

Desktop window:

```bash
dx serve --desktop
```

Tests (parsing, spelling, layout):

```bash
cargo test --no-default-features
```

Contact sheet of arbitrary syllables as static HTML (handy for tuning glyphs).
The last argument is the brush, from 0 (geometric) to 1 (full brush):

```bash
cargo run --example sheet --no-default-features -- sheet.html "zhuāng xiǎng|yú lǜ" 120 0.5
```

Regenerate the character data from Unicode's Unihan database:

```bash
python3 tools/gen_hanzi.py path/to/unzipped/Unihan data
```

## Layout

| file | what |
|------|------|
| `src/phonology.rs` | syllable model in zhuyin terms (initial, medial, rhyme, tone), pinyin and zhuyin spelling |
| `src/parse.rs` | input parsing: hanzi (via the `pinyin` crate + phrase table), pinyin segmentation, zhuyin, tone sandhi |
| `src/glyph.rs` | stroke definitions for every component and the block layout |
| `src/brush.rs` | brush simulator: turns stroke centre-lines into filled, handwritten-looking outlines |
| `src/hanzi.rs` | stroke counts and same-reading characters, from `data/` |
| `src/keyboard.rs` | on-screen keyboard logic: which keys can follow, composing and committing |
| `src/main.rs` | Dioxus UI: input, keyboard, brush controls, output, inspector, reference key |
| `assets/main.css` | styling (light and dark) |
| `data/` | character data generated from Unihan (`tools/gen_hanzi.py`) |

## Data

`data/hanzi.txt` and `data/strokes_cjk.txt` are derived from the Unicode
Unihan database (kTotalStrokes, kHanyuPinlu, kTGHZ2013), © Unicode, Inc.,
used under the [Unicode License v3](https://www.unicode.org/license.txt).
Hanzi input is read with the [`pinyin`](https://crates.io/crates/pinyin)
crate.
