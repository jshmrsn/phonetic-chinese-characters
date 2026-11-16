"""Build data/words.txt, a word-frequency list for segmenting hanzi into words.

Source: jieba's dictionary (MIT License, see data/LICENSE-jieba.txt).
Usage: download https://raw.githubusercontent.com/fxsjy/jieba/master/jieba/dict.txt, then
    python3 tools/gen_words.py dict.txt data/words.txt
Keeps all-hanzi entries of 1-8 characters; multi-character words need a
frequency of at least 10. The first line holds the frequency total of the
whole source dictionary, used to turn counts into probabilities.
"""
import sys

MIN_FREQ = 10

def han(c):
    return 0x3400 <= ord(c) <= 0x9FFF or 0xF900 <= ord(c) <= 0xFAFF or 0x20000 <= ord(c) <= 0x3134F

src, out = sys.argv[1], sys.argv[2]
total = 0
keep = []
for line in open(src, encoding="utf-8"):
    parts = line.split()
    if len(parts) < 2:
        continue
    word, freq = parts[0], int(parts[1])
    total += freq
    if all(han(c) for c in word) and 1 <= len(word) <= 8 and (freq >= MIN_FREQ or len(word) == 1):
        keep.append((word, freq))
with open(out, "w", encoding="utf-8") as f:
    f.write(f"# word frequencies from the jieba dictionary (MIT License, see LICENSE-jieba.txt)\n")
    f.write(f"total\t{total}\n")
    for word, freq in keep:
        f.write(f"{word}\t{freq}\n")
print(len(keep), "words")
