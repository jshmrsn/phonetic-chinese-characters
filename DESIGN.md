# 音块 Yīnkuài: design and ruleset

A phonetic script for Mandarin in which **every syllable is one square block**,
like a Chinese character. Blocks are assembled from a small set of reusable
components (initials, vowels, codas, tone marks) following fixed rules, in the
spirit of Hangul. The components are drawn from the strokes and components of
Chinese writing.

Design goals:

1. **One block = one syllable**, and a block never depends on its neighbours.
   Tone sandhi is the only cross-syllable effect in Mandarin. The script writes
   the citation tone, as dictionaries and pinyin do.
2. **Featural**: sounds that are made alike look alike. All consonants made at
   the same place share a frame, and all consonants made the same way share a
   mark.
3. **Chinese-looking**: only the classical strokes (横 竖 撇 点 捺 提 and their
   折 / 钩 / 弯 compounds) are used, and most components are real ones (口 厂 匚
   冂 勹 乚 大 又 了 乙 土 儿).
4. **Phonemic spelling**: one component per sound, with none of pinyin's
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
│        │      coda      │   -i -u -n -ng (flattened), and/or 儿
└────────┴────────────────┘
```

| Syllable has…              | Structure                          | Example            |
|----------------------------|------------------------------------|--------------------|
| initial + final            | left–right, like 好: initial on the left (≈⅓), final stacked on the right | 好 hǎo, 装 zhuāng |
| final only (no consonant)  | the final stack takes the full width | 有 yǒu, 我 wǒ, 鱼 yú |
| initial only               | the initial fills the block (zi ci si zhi chi shi ri, as in zhuyin) | 是 shì, 字 zì |
| syllabic nasal             | the nasal component alone          | 嗯 ńg, 呣 m̀         |

The final is stacked top to bottom (like the right half of 漫 or the whole of 意):
medial, then vowel, then coda. Glides and codas are **flattened** forms, the way
艹 sits on top and 灬 sits below, so the vowel is always the visual centre.

When a syllable has both a coda and 儿 (e.g. diǎnr), they share the coda row
side by side.

## 2. Initials: frame × mark

Picture a mouth **facing left**. The frame is an enclosure showing *where* the
closure is made. The mark inside shows *how* the air escapes.

| place ↓ / manner → | (none) unaspirated | 一 aspirated | 丶 nasal | 乂 fricative | 丨 liquid |
|--------------------|-----------|-------|-------|-------|-------|
| **口** labial: fully closed box, lips shut | b | p (=日) | m | f | |
| **厂** alveolar: closed top-front, tongue tip on the gum ridge | d | t | n | | l |
| **匚** dental: closed top, front and bottom, teeth together | z | c | | s (=区) | |
| **乚** retroflex: hook curling up, tongue tip curled back | zh | ch | | sh | r |
| **冂** palatal: roof over everything, tongue body on the palate | j | q | | x (=冈) | |
| **勹** velar: closed top-back, tongue root on the soft palate | g | k | ng (=勺) | h | |

The marks:

* **(nothing)**: a clean stop or affricate.
* **一** aspirated: one level stroke, a puff of breath (p t c ch q k).
* **丶** nasal: a dot, the voice resonating in the nose (m n ng).
* **乂** fricative: crossed strokes, turbulent air (f s sh x h).
* **丨** liquid: one upright stroke, the tongue held in place while air flows round it (l r).

So learning 6 frames and 4 marks is enough to read all 21 initials plus ng.

## 3. Vowels

Vowels are **open** shapes and consonants are **enclosures**, so the two classes
never look alike.

| vowel | component | mnemonic / idea | glide form |
|-------|-----------|-----------------|------------|
| a | 大 | dà: the mouth opened wide | — |
| o | 又 | the looped stroke of rounded lips | — |
| e | 了 | le: the most common neutral e | — |
| ê | 了 + 丶 | e with a dot, like pinyin's circumflex | — |
| i | 乙 | yǐ | flattened 乙 |
| u | 土 | tǔ | flattened 土 |
| ü | 土 + 丷 | u plus two dots, exactly like pinyin's umlaut | flattened |

i, u and ü have flattened *glide* forms. On top they are the medial (gliding
in), at the bottom they are a coda (gliding out). **Position tells glide from
vowel; shape tells which sound.**

## 4. Codas

The coda reuses components the reader already knows:

| coda | written as | why |
|------|-----------|-----|
| -i (ai ei uai uei) | flattened 乙 | the vowel i, gliding out |
| -u (ao ou iao iou) | flattened 土 | the vowel u, gliding out (pinyin's *ao* is a + u) |
| -n | 厂 + 丶 | identical to the initial n |
| -ng | 勹 + 丶 (勺) | the velar nasal: velar frame + nasal mark |
| -r | 儿 | 儿化, and the r of er (er = e + 儿) |

Because -n and -ng differ by *frame* (alveolar 厂 vs velar 勹), the script shows
exactly the articulatory difference that learners find hard to hear.

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
| er | e + 儿 | the same 儿 as erhua |
| y-, w- | medial i / u | y and w are just glides |
| zi ci si zhi chi shi ri | initial alone | no vowel component |
| ong, iong | o · ng, i · o · ng | as in pinyin |

The full table of finals (medial · vowel · coda):

```
a   ·a·      o   ·o·      e   ·e·      ê   ·ê·      er  ·e·儿
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
| initial (beside a final) | 4, 21, 33, 76 | 0.40–0.86 (口 kept ≥ 0.62 and set slightly high, as in 吃) |
| initial (alone) | 12, 22, 76, 74 | 0.66–0.92 |
| final (beside an initial) | 43, 21, 53, 76 | — |
| final (alone) | 8, 21, 84, 76 | — |

Inside the final column the parts share the height by weight: medial 0.34
(ü 0.42), vowel 0.62, coda 0.36 (0.42 when it holds both a coda and 儿), with
5.5 units between them. Glides use aspect 1.6–3.4, nasal codas 1.3–2.6, and a
stacked vowel 0.7–1.7.

## 8. Known trade-offs

* **乙 (i) and 乚 (zh)** share a bottom curve. They differ in their top, a
  horizontal-plus-diagonal against a plain vertical, and in position: i is
  never on the left.
* **一 and 丶 inside a narrow frame** (p vs m, t vs n) are small at small
  sizes. They differ in direction (level vs slanted).
* Some blocks coincide with real characters: s = 区, x = 冈, ng = 勺, p = 日,
  and ā looks a little like 天. Japanese kana live with the same kind of
  overlap (カ/力, ロ/口).
* *bo po mo fo* are written with plain o, as in pinyin and zhuyin, although
  they are pronounced [uo].
* Hanzi input is read character by character from a dictionary, plus a small
  table of common context-dependent words. Pinyin and zhuyin input are exact.
