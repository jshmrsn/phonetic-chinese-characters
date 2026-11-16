//! Best-guess word segmentation for hanzi, the way jieba does it: every way of
//! cutting a run of characters into dictionary words is scored by the product
//! of the words' probabilities, and the best cut wins (dynamic programming
//! over the run). Characters the dictionary doesn't know stand alone.
//!
//! The word list (`data/words.txt`) comes from jieba's dictionary, which is
//! for simplified Chinese.

use std::collections::HashMap;
use std::sync::OnceLock;

static WORDS: &str = include_str!("../data/words.txt");

struct Dict {
    freq: HashMap<&'static str, u32>,
    log_total: f64,
    max_len: usize,
}

fn dict() -> &'static Dict {
    static DICT: OnceLock<Dict> = OnceLock::new();
    DICT.get_or_init(|| {
        let mut freq = HashMap::new();
        let mut total = 1.0f64;
        let mut max_len = 1;
        for line in WORDS.lines().filter(|l| !l.starts_with('#')) {
            let Some((word, n)) = line.split_once('\t') else { continue };
            let Ok(n) = n.parse::<u64>() else { continue };
            if word == "total" {
                total = n as f64;
                continue;
            }
            max_len = max_len.max(word.chars().count());
            freq.insert(word, n as u32);
        }
        Dict { freq, log_total: total.ln(), max_len }
    })
}

/// Split a run of hanzi into words. Returns the length of each word in
/// characters; the lengths add up to `chars.len()`.
pub fn segment(chars: &[char]) -> Vec<usize> {
    let n = chars.len();
    let d = dict();
    // best[i]: (log-probability of the best cut of chars[i..], first word length)
    let mut best = vec![(0.0f64, 0usize); n + 1];
    let mut word = String::new();
    for i in (0..n).rev() {
        let mut top = (f64::NEG_INFINITY, 1);
        word.clear();
        for j in i..n.min(i + d.max_len) {
            word.push(chars[j]);
            let len = j - i + 1;
            let log_p = match d.freq.get(word.as_str()) {
                Some(&f) => (f as f64).ln() - d.log_total,
                // an unknown character counts as seen once
                None if len == 1 => -d.log_total,
                None => continue,
            };
            let score = log_p + best[j + 1].0;
            if score > top.0 {
                top = (score, len);
            }
        }
        best[i] = top;
    }
    let mut out = Vec::new();
    let mut i = 0;
    while i < n {
        out.push(best[i].1);
        i += best[i].1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(s: &str) -> Vec<String> {
        let chars: Vec<char> = s.chars().collect();
        let mut out = Vec::new();
        let mut i = 0;
        for len in segment(&chars) {
            out.push(chars[i..i + len].iter().collect());
            i += len;
        }
        out
    }

    #[test]
    fn segments_common_sentences() {
        assert_eq!(words("我们一起学习中文"), ["我们", "一起", "学习", "中文"]);
        assert_eq!(words("图书馆正式向市民开放"), ["图书馆", "正式", "向", "市民", "开放"]);
        assert_eq!(words("我去银行"), ["我", "去", "银行"]);
    }

    #[test]
    fn unknown_characters_stand_alone() {
        assert_eq!(segment(&['㐀', '㐁']), [1, 1]);
        assert!(segment(&[]).is_empty());
    }
}
