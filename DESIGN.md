# 音块 Yīnkuài: design and ruleset

A phonetic script for Mandarin in which **every syllable is one square block**,
like a Chinese character. Blocks are assembled from a small set of reusable
components (initials, vowels, codas, tone marks) following fixed rules, in the
spirit of Hangul. Every component is drawn with the strokes of Chinese
writing.

Design goals:

1. **One block = one syllable**, and a block never depends on its neighbours.
   Tone sandhi is the only cross-syllable effect in Mandarin. The script writes
   the citation tone, as dictionaries and pinyin do.
2. **Minimal**: every component is one or two strokes, plus at most one mark.
   A typical block has 4–8 strokes.
3. **Featural**: sounds that are made alike look alike. All consonants made at
   the same place share a frame, all consonants made the same way share a mark,
   and a dot always means "front vowel".
4. **Chinese strokes only**: 横 竖 撇 点 捺 提 and their 折 / 钩 / 弯 compounds,
   at one constant weight. Components borrow the *look* of familiar components
   (匚 厂 凵 乚 冂 八 又 了 乙 乂) without having to be complete characters.
5. **Phonemic spelling**: one component per sound, with none of pinyin's
   abbreviations.

---

## 1. The block

```
┌─────────────────────────┐
│          tone           │   top centre, always
├────────┬────────────────┤
│        │     medial     │   i / u / ü glide (flattened vowel)
│ initial│────────────────│
│        │     vowel      │   the nucleus (full-size vowel)
│        │────────────────│
│        │      coda      │   flattened -i -u, or a bare frame for -n -ng -r
└────────┴────────────────┘
```

| Syllable has…              | Structure                          | Example            |
|----------------------------|------------------------------------|--------------------|
| initial + final            | left–right, like 好: initial on the left (≈⅓), final stacked on the right | 好 hǎo, 装 zhuāng |
| final only (no consonant)  | the final stack takes the full width | 有 yǒu, 我 wǒ, 鱼 yú |
| initial only               | the initial fills the block (zi ci si zhi chi shi ri, as in zhuyin) | 是 shì, 字 zì |
| syllabic nasal             | the full nasal consonant alone     | 嗯 ńg, 呣 m̀         |

The final is stacked top to bottom (like the right half of 漫 or the whole of 意):
medial, then vowel, then coda. Glides and codas are **flattened**, the way 艹
sits on top and 灬 sits below, so the vowel is always the visual centre.

When a syllable has both a coda and r-colouring (e.g. diǎnr), they share the
coda row side by side.

## 2. Initials: frame × mark

Picture a mouth **facing left**: lips on the left, the roof of the mouth on
top, the throat on the right. The frame is drawn along the surface where the
closure is made. A single mark inside shows *how* the air escapes.

| place ↓ / manner → | (none) unaspirated | 一 aspirated | 丶 nasal | 丿 fricative | 丨 liquid |
|--------------------|-----------|-------|-------|-------|-------|
| **匚** labial: closed at the front, the lips | b | p | m | f | |
| **厂** alveolar: closed top-front, tongue tip on the gum ridge | d | t | n | | l |
| **凵** dental: closed below, tongue tip down behind the lower teeth | z | c | | s | |
| **乚** retroflex: one rising hook, tongue tip curled up | zh | ch | | sh | r |
| **冂** palatal: a roof over everything, tongue body on the palate | j | q | | x | |
| **𠃌** velar: one 横折钩, closed top-back at the soft palate | g | k | ng | h | |

The marks, each exactly one stroke:

* **(nothing)**: a clean stop or affricate.
* **一** aspirated: a level stroke, a puff of breath (p t c ch q k).
* **丶** nasal: a dot, the voice resonating in the nose (m n ng).
* **丿** fricative: a slash, air squeezed through a narrow gap (f s sh x h).
* **丨** liquid: an upright stroke, the tongue held in place while air flows round it (l r).

Frames are one or two strokes (乚 and 𠃌 are single strokes), so no initial has
more than three. Six frames and four marks are enough to read all 21 initials
plus ng.

The frames for labial, dental and palatal are the same three-sided box opened
toward the right, the top and the bottom (匚 凵 冂). Each one is closed on the
side where the mouth closes.

## 3. Vowels

Vowels are free-standing strokes and consonants are frames, so the two classes
never look alike.

| vowel | form | strokes | idea | glide form |
|-------|------|---------|------|------------|
| a | 八 | 2 | bā: lips and jaw spread wide open | — |
| o | 又 | 2 | the crossing of u under a lid: rounded, but more open | — |
| e | 了 | 2 | le: the neutral e | — |
| ê | 了 + 丶 | 3 | e + fronting dot (pinyin ê) | — |
| i | 乙 | 1 | yǐ | flattened 乙 |
| u | 乂 | 2 | zhuyin's ㄨ: crossed strokes for pursed lips | flattened 乂 |
| ü | 乂 + 丶 (义) | 3 | u + fronting dot (pinyin ü) | flattened 义 |

**A dot fronts a vowel**: ê is the front counterpart of e, and ü is the front
counterpart of u. This is the role pinyin's ^ and ¨ play.

i, u and ü have flattened *glide* forms. On top they are the medial (gliding
in), at the bottom they are a coda (gliding out). **Position tells glide from
vowel; shape tells which sound.** Crossed glides (乂, 义) are never flattened
past 2.3 : 1, so the cross stays readable.

## 4. Codas

A Mandarin syllable can only end in a glide (-i, -u) or a sonorant (-n, -ng, -r).
Because nothing else can stand there, the coda drops every mark:

| coda | written as | why |
|------|-----------|-----|
| -i (ai ei uai uei) | flattened 乙 | the vowel i, gliding out |
| -u (ao ou iao iou) | flattened 乂 | the vowel u, gliding out (pinyin's *ao* is a + u) |
| -n | bare 厂 | the alveolar frame: tongue tip up front |
| -ng | bare 𠃌 | the velar frame: tongue root up at the back |
| -r | bare 乚 | the retroflex frame: tongue tip curled (儿化, and the r of er) |
| -m (呣 hm only) | bare 匚 | the labial frame |

-n and -ng differ only in frame, front 厂 vs back 𠃌, which is exactly the
articulatory difference learners find hard to hear. A nasal with no vowel
(嗯 ńg, 呣 m̀) is syllabic, and is written as the full consonant, mark included.

## 5. Tones

The tone mark sits at the top centre of the block. It uses **pinyin's own four
marks**, drawn as brush strokes:

| tone | pinyin | stroke |
|------|--------|--------|
| 1 | ¯ | 横, short level stroke |
| 2 | ´ | 提, rising from lower-left to upper-right |
| 3 | ˇ | 点 + 提, down then up |
| 4 | ` | 点, falling from upper-left to lower-right |
| neutral | — | no mark |

## 6. Spelling rules

The script spells what is pronounced, undoing pinyin's shortcuts:

| pinyin | written as | note |
|--------|-----------|------|
| iu | i · o · u | 六 liù = l + i/o/u |
| ui | u · e · i | 贵 guì = g + u/e/i |
| un | u · e · n | 顿 dùn = d + u/e/n |
| ju qu xu yu | j · ü … | the ü is always written |
| ie, üe | i · ê, ü · ê | the open e |
| ao, iao | a · u, i · a · u | the coda is u |
| er | e + r | the same r coda as 儿化 |
| y-, w- | medial i / u | y and w are just glides |
| zi ci si zhi chi shi ri | initial alone | no vowel component |
| ong, iong | o · ng, i · o · ng | as in pinyin |

The full table of finals (medial · vowel · coda):

```
a   ·a·      o   ·o·      e   ·e·      ê   ·ê·      er  ·e·r
ai  ·a·i     ei  ·e·i     ao  ·a·u     ou  ·o·u
an  ·a·n     en  ·e·n     ang ·a·ng    eng ·e·ng    ong ·o·ng
i   ·i·      ia  i·a·     ie  i·ê·     iao i·a·u    iou i·o·u
ian i·a·n    in  ·i·n     iang i·a·ng  ing ·i·ng    iong i·o·ng
u   ·u·      ua  u·a·     uo  u·o·     uai u·a·i    uei u·e·i
uan u·a·n    uen u·e·n    uang u·a·ng  ueng u·e·ng
ü   ·ü·      üe  ü·ê·     üan ü·a·n    ün  ·ü·n
```

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

Inside the final column the parts share the height by weight: medial 0.34
(ü 0.42), vowel 0.62, coda 0.32 (0.4 when it holds both a coda and r), with
5.5 units between them. Glides use aspect 1.6–3.4 (crossed glides 1.6–2.3),
bare-frame codas 1.5–3.0, and a stacked vowel 0.7–1.7.

## 8. Known trade-offs

* **乙 (i) and 乚 (zh, and the r coda)** share a bottom curve. They differ in
  their top, a horizontal-plus-diagonal against a plain vertical, and in
  position.
* **匚 凵 冂** are rotations of one box. That is the point (closed side =
  place of closure), but it asks the reader to notice orientation.
* **一 and 丶 inside a narrow frame** (p vs m, t vs n) are small at small
  sizes. They differ in direction (level vs slanted).
* Some blocks coincide with real characters or components: 七-like ch, 勹-like
  h, and ü looks like 义. Japanese kana live with the same kind of overlap
  (カ/力, ロ/口).
* *bo po mo fo* are written with plain o, as in pinyin and zhuyin, although
  they are pronounced [uo].
* Hanzi input is read character by character from a dictionary, plus a small
  table of common context-dependent words. Pinyin and zhuyin input are exact.
