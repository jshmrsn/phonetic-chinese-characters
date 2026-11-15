# 音块 Yīnkuài

A featural, one-block-per-syllable phonetic script for Mandarin, with a Rust +
Dioxus app that converts Hanzi, pinyin (tone marks or numbers) or zhuyin into
the script.

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

Contact sheet of arbitrary syllables as static HTML (handy for tuning glyphs):

```bash
cargo run --example sheet --no-default-features -- sheet.html "zhuāng xiǎng|yú lǜ" 120
```

## Layout

| file | what |
|------|------|
| `src/phonology.rs` | syllable model in zhuyin terms (initial, medial, rhyme, tone), pinyin and zhuyin spelling |
| `src/parse.rs` | input parsing: hanzi (via the `pinyin` crate + phrase table), pinyin segmentation, zhuyin, tone sandhi |
| `src/glyph.rs` | stroke definitions for every component and the block layout |
| `src/main.rs` | Dioxus UI: input, rendered output, inspector, reference key |
| `assets/main.css` | styling (light and dark) |
