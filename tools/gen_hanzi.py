"""Build data/hanzi.txt and data/strokes_cjk.txt from the Unicode Unihan database.

Usage: download and unzip https://www.unicode.org/Public/UCD/latest/ucd/Unihan.zip, then
    python3 tools/gen_hanzi.py <unzipped Unihan dir> data
"""
import re, sys
src, out_dir = sys.argv[1], sys.argv[2]

def fields(name):
    d = {}
    with open(f"{src}/{name}", encoding="utf-8") as f:
        for line in f:
            if line.startswith("#") or not line.strip():
                continue
            cp, key, val = line.rstrip("\n").split("\t", 2)
            d.setdefault(key, {})[int(cp[2:], 16)] = val
    return d

readings = fields("Unihan_Readings.txt")
irg = fields("Unihan_IRGSources.txt")
strokes = {cp: int(v.split()[0]) for cp, v in irg["kTotalStrokes"].items()}
pinlu = readings["kHanyuPinlu"]
tghz = readings.get("kTGHZ2013", {})

chars = sorted(set(pinlu) | set(tghz))
lines = ["# Hanzi data, generated from the Unicode Unihan database",
         "# (kTotalStrokes, kHanyuPinlu, kTGHZ2013). Unicode License v3,",
         "# https://www.unicode.org/license.txt",
         "# char<TAB>strokes<TAB>reading[:frequency] ..."]
for cp in chars:
    rs = []
    for item in pinlu.get(cp, "").split():
        m = re.match(r"^(.+)\((\d+)\)$", item)
        if m:
            rs.append((m.group(1), int(m.group(2))))
    seen = {r for r, _ in rs}
    for item in tghz.get(cp, "").split():
        r = item.split(":", 1)[1]
        if r not in seen:
            seen.add(r)
            rs.append((r, 0))
    if not rs:
        continue
    rd = " ".join(f"{r}:{n}" if n else r for r, n in rs)
    lines.append(f"{chr(cp)}\t{strokes.get(cp, 0)}\t{rd}")
open(f"{out_dir}/hanzi.txt", "w", encoding="utf-8").write("\n".join(lines) + "\n")

# one byte per code point U+4E00..U+9FFF: chr(48 + strokes), '0' if unknown
table = "".join(chr(48 + min(strokes.get(cp, 0), 78)) for cp in range(0x4E00, 0xA000))
open(f"{out_dir}/strokes_cjk.txt", "w", encoding="ascii").write(table)
print(len(lines) - 4, "characters;", len(table), "stroke entries")
