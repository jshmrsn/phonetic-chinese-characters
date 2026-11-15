# Phonetic Chinese Characters Brainstorming

An experiment with a phonetic script for Mandarin where each syllable is
written as one square block. A block is assembled from a few stroke-based
parts (initial, medial ㄧㄨㄩ, rhyme, tone), following zhuyin's analysis of
the syllable. It is a brainstorm, not a tested or finished system.

The app (Rust + Dioxus) renders hanzi, pinyin or zhuyin in the script, and
includes a mock-up of an on-screen keyboard.

![Text tab](screenshots/overview.png)

![Keyboard tab](screenshots/keyboard-fixed.png)

The keyboard can also hide the keys that can't continue the current syllable:

![Dynamic keyboard](screenshots/keyboard-dynamic.png)

![Initials](screenshots/initials.png)

![Rhymes](screenshots/rhymes.png)

| Geometric strokes | Brush-like strokes |
|---|---|
| ![](screenshots/brush-geometric.png) | ![](screenshots/brush-handwritten.png) |

## Running

```bash
dx serve --web
```

```bash
cargo test --no-default-features
```

Design notes are in [DESIGN.md](DESIGN.md). Hanzi readings come from the
`pinyin` crate. Stroke counts and reading frequencies come from the Unicode
Unihan database (`data/`, Unicode License v3).
