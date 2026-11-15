# 音块 Yīnkuài: design and ruleset

A phonetic script for Mandarin in which **every syllable is one square block**,
like a Chinese character. A block is assembled from a small set of components
following fixed rules, in the spirit of Hangul. Every component is drawn with
the strokes of Chinese writing.

The script follows **zhuyin's analysis of Mandarin, not pinyin's spelling**.
A syllable is an initial, a medial glide and a rhyme, plus the tone. Each of
those is one symbol, and each symbol stands for one sound.

Design goals:

1. **One block = one syllable**, and a block never depends on its neighbours.
   Tone sandhi is the only cross-syllable effect in Mandarin. The script writes
   the citation tone, as dictionaries do.
2. **One symbol per sound**: a diphthong or nasal rhyme such as ㄠ ao or ㄤ ang is
   a single symbol, not a string of letters. Sounds that are related share a
   base (ㄚ a → ㄞ ㄠ ㄢ ㄤ). Sounds that are different never share a shape,
   even where pinyin uses the same letter: ㄜ (me), ㄟ (mei) and ㄝ (ye) look
   unrelated.
3. **Nothing hidden**: no dropped vowels (liu is ㄌㄧㄡ, gui is ㄍㄨㄟ, dun is
   ㄉㄨㄣ), ü is always ü (ju is ㄐㄩ), and no Latin letters to mislead.
4. **Featural and minimal**: consonants share a frame per place and a mark per
   manner; rhymes share an ending stroke per ending. Every symbol is one to
   three strokes.
5. **Chinese strokes only**: 横 竖 撇 点 捺 提 and their 折 / 钩 / 弯 compounds,
   at one constant weight.

---

## 1. The block

```
┌─────────────────────────┐
│          tone           │   top centre, always
├────────┬────────────────┤
│        │     medial     │   ㄧ ㄨ ㄩ, flattened
│ initial│────────────────│
│        │                │
│        │      rhyme     │   one symbol: vowel + at most one ending stroke
│        │                │
└────────┴────────────────┘
```

| Syllable has…              | Structure                          | Example            |
|----------------------------|------------------------------------|--------------------|
| initial + final            | left–right, like 好: initial on the left (≈⅓), final on the right | 好 ㄏㄠ, 装 ㄓㄨㄤ |
| final only (no consonant)  | the final takes the full width     | 有 ㄧㄡ, 我 ㄨㄛ, 鱼 ㄩ |
| initial only               | the initial fills the block (zi ci si zhi chi shi ri, as in zhuyin) | 是 ㄕ, 字 ㄗ |
| syllabic nasal             | the full nasal consonant alone     | 嗯 ㄫ, 呣 ㄇ         |

The final is at most two parts: an optional medial flattened on top, the way
艹 sits on a character, and the rhyme beneath it. With no medial, the rhyme
fills the column.

## 2. Initials: frame × mark

Picture a mouth **facing left**: lips on the left, the roof of the mouth on
top, the throat on the right. The frame is drawn along the surface where the
closure is made. A single mark inside shows *how* the air escapes.

| place ↓ / manner → | (none) unaspirated | 一 aspirated | 丶 nasal | 丿 fricative | 丨 liquid |
|--------------------|-----------|-------|-------|-------|-------|
| **匚** labial: closed at the front, the lips | ㄅ b | ㄆ p | ㄇ m | ㄈ f | |
| **厂** alveolar: closed top-front, tongue tip on the gum ridge | ㄉ d | ㄊ t | ㄋ n | | ㄌ l |
| **凵** dental: closed below, tongue tip down behind the lower teeth | ㄗ z | ㄘ c | | ㄙ s | |
| **乚** retroflex: one rising hook, tongue tip curled up | ㄓ zh | ㄔ ch | | ㄕ sh | ㄖ r |
| **冂** palatal: a roof over everything, tongue body on the palate | ㄐ j | ㄑ q | | ㄒ x | |
| **𠃌** velar: one 横折钩, closed top-back at the soft palate | ㄍ g | ㄎ k | ㄫ ng | ㄏ h | |

The marks, each exactly one stroke:

* **(nothing)**: a clean stop or affricate.
* **一** aspirated: a level stroke, a puff of breath.
* **丶** nasal: a dot, the voice resonating in the nose.
* **丿** fricative: a slash, air squeezed through a narrow gap.
* **丨** liquid: an upright stroke, the tongue held in place while air flows round it.

## 3. Rhymes: thirteen symbols

These are zhuyin's thirteen rhymes. Each is one symbol: a **vowel** plus at most
one **ending stroke**. Rhymes on the same vowel sound share the vowel. The
ending stroke is the same for every vowel and is written close under it, so
vowel and ending read as one shape.

| vowel ↓ / ending → | alone | 提 -i | 卧钩 -u | 横 -n | 横折钩 -ng | 竖弯钩 -r |
|--------------------|-------|-------|--------|-------|-----------|----------|
| **八** ㄚ a, the jaw wide open | ㄚ a | ㄞ ai | ㄠ ao | ㄢ an | ㄤ ang | |
| **又** ㄛ o, a rounded loop | ㄛ o | | ㄡ ou | | | |
| **了** ㄜ e, unrounded mid (哥 根 冷) | ㄜ e | | | ㄣ en | ㄥ eng | ㄦ er |
| **人** ㄝ ê, front mid (也 谢 美) | ㄝ ê | ㄟ ei | | | | |

Why these groupings:

* **ㄟ ei is built on ㄝ ê, not on ㄜ e.** The vowel of 美 měi [meɪ] is the front
  mid vowel of 也 yě [jɛ], while 么 me [mɤ] is a back vowel. Pinyin writes all
  three with "e"; here they are 人+提, 人 and 了.
* ㄣ en [ən], ㄥ eng [ɤŋ] and ㄦ er [ɚ] share the unrounded mid vowel of ㄜ, so
  they are built on 了.
* 人 (ê) is 八 (a) closed at the top: the jaw half shut.

The ending strokes:

| ending | stroke | idea |
|--------|--------|------|
| -i | 提 ㇀ | rising: the tongue rises toward i |
| -u | 卧钩 ㇃ | a bowl: the lips round toward u |
| -n | 横 一 | a level bar: the tongue tip seals the mouth |
| -ng | 横折钩 𠃌 | the same bar closed at the back, like the velar frame |
| -r | 竖弯钩 乚 | the retroflex curl |

**儿化** adds the -r curl to any rhyme: huār ㄏㄨㄚㄦ is 八 with the curl, and
diǎnr puts the bar and the curl side by side.

## 4. Medials ㄧ ㄨ ㄩ

| glide | form | idea |
|-------|------|------|
| ㄧ i | 乙 | yǐ, one stroke |
| ㄨ u | 乂 | zhuyin's own ㄨ, crossed strokes for pursed lips |
| ㄩ ü | 乂 + 丶 (义) | u with the tongue pushed forward (the dot) |

When a rhyme follows, the glide is flattened on top of it. When the glide is
the whole final (yi, wu, yu, ji, bu, lü …), it is written full size. Crossed
glides (乂, 义) are never flattened past 2.3 : 1, so the cross stays readable.

## 5. Tones

The tone mark sits at the top centre of the block. It uses pinyin's four
marks, drawn as brush strokes:

| tone | mark | stroke |
|------|------|--------|
| 1 | ¯ | 横, short level stroke |
| 2 | ´ | 提, rising from lower-left to upper-right |
| 3 | ˇ | 点 + 提, down then up |
| 4 | ` | 点, falling from upper-left to lower-right |
| neutral | — | no mark |

## 6. Every final

A final is a medial, a rhyme, or a medial + rhyme, exactly as zhuyin writes it:

```
ㄚ a     ㄛ o     ㄜ e     ㄝ ê     ㄞ ai    ㄟ ei    ㄠ ao    ㄡ ou
ㄢ an    ㄣ en    ㄤ ang   ㄥ eng   ㄦ er
ㄧ i     ㄧㄚ ia   ㄧㄝ ie   ㄧㄠ iao  ㄧㄡ iu   ㄧㄢ ian  ㄧㄣ in   ㄧㄤ iang  ㄧㄥ ing
ㄨ u     ㄨㄚ ua   ㄨㄛ uo   ㄨㄞ uai  ㄨㄟ ui   ㄨㄢ uan  ㄨㄣ un   ㄨㄤ uang  ㄨㄥ ong / weng
ㄩ ü     ㄩㄝ üe   ㄩㄢ üan  ㄩㄣ ün   ㄩㄥ iong
(none) the buzzing -i of zi ci si zhi chi shi ri
```

Note how pinyin's shortcuts disappear: in, ing, un, ün, ong and iong are a
medial plus the rhymes ㄣ and ㄥ, and iu and ui show their middle vowel.

## 7. Assembly geometry

The em box is 100 × 100. Strokes are centre-lines with round caps and joins at
a constant width (5.6, tone marks 4.6), a rounded-Hei (圆体) style. Every
component is designed in a unit box and mapped into its slot. Each slot allows
a range of aspect ratios, and a component that would be stretched past it is
shrunk and centred instead. The stroke width is applied after mapping, so a
flattened component keeps its weight.

| slot | rectangle (x, y, w, h) | aspect range (w/h) |
|------|------------------------|--------------------|
| tone | centred at (50, 9), ~20 × 12 | fixed |
| initial (beside a final) | 4, 21, 33, 76 | 0.40–0.86 |
| initial (alone) | 12, 22, 76, 74 | 0.66–0.92 |
| final (beside an initial) | 43, 21, 53, 76 | — |
| final (alone) | 8, 21, 84, 76 | — |

The medial and rhyme share the final's height by weight: medial 0.34 (ㄩ 0.42),
rhyme 0.62 (0.95 when it has an ending stroke), with 6 units between them.
Inside a rhyme the ending takes the bottom 30%, only 3% below the vowel, so the
two read as one symbol. Rhyme aspect windows are 0.7–1.7 under a medial,
0.5–1.0 beside an initial alone, and 0.7–1.06 filling the block.

## 8. Known trade-offs

* **乙 (ㄧ) and 乚 (zh, and the -r curl)** share a bottom curve. They differ in
  their top, a horizontal-plus-diagonal against a plain vertical, and in
  position.
* **八 (ㄚ) and 人 (ㄝ)** differ only in whether the two strokes meet at the
  top. That mirrors the sounds (open vs half-open jaw), but they must be kept
  well apart in small sizes.
* **匚 凵 冂** are rotations of one box. That is the point (closed side =
  place of closure), but it asks the reader to notice orientation.
* **一 and 丶 inside a narrow frame** (p vs m, t vs n) are small at small
  sizes. They differ in direction (level vs slanted).
* *bo po mo fo* are written with ㄛ, as in zhuyin, although they are
  pronounced [uo].
* Hanzi input is read character by character from a dictionary, plus a small
  table of common context-dependent words. Pinyin and zhuyin input are exact.
