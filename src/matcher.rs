use crate::config::ErrorEntry;
use crate::input::ParsedError;
use serde::Serialize;

/// Result of error remapping
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemapResult {
    /// Remapped error code
    pub code: String,
    /// Remapped error description
    pub custom_desc: String,
    /// Whether a match was found in the dictionary
    pub matched: bool,
}

/// A scored match candidate
#[derive(Debug)]
struct ScoredEntry<'a> {
    entry: &'a ErrorEntry,
    score: f64,
}

/// Perform case-insensitive substring check.
/// Returns true if `needle` is contained within `haystack`.
fn contains_substring_ci(haystack: &str, needle: &str) -> bool {
    let h = haystack.to_lowercase();
    let n = needle.to_lowercase();
    h.contains(&n)
}

/// Compute a fuzzy similarity score between a substring pattern and a text.
///
/// Strategy:
/// 1. Exact substring containment (case-insensitive) → score 1.0
/// 2. Sliding window normalized Levenshtein over the text with window size
///    equal to the pattern length → best score from all windows
fn fuzzy_score(text: &str, pattern: &str) -> f64 {
    if pattern.is_empty() || text.is_empty() {
        return 0.0;
    }

    // Case-insensitive exact substring match
    if contains_substring_ci(text, pattern) {
        return 1.0;
    }

    let text_lower = text.to_lowercase();
    let pattern_lower = pattern.to_lowercase();

    // Use normalized Levenshtein on the full strings as a baseline
    let full_score = strsim::normalized_levenshtein(&text_lower, &pattern_lower);

    // Sliding window approach: compare pattern against each window of the text
    // with window size = pattern length (in chars)
    let text_chars: Vec<char> = text_lower.chars().collect();
    let pattern_chars: Vec<char> = pattern_lower.chars().collect();
    let pattern_len = pattern_chars.len();

    if text_chars.len() < pattern_len {
        // Text is shorter than pattern — just compare directly
        return full_score;
    }

    let mut best_window_score = 0.0f64;
    for start in 0..=(text_chars.len() - pattern_len) {
        let window: String = text_chars[start..start + pattern_len].iter().collect();
        let score = strsim::normalized_levenshtein(&window, &pattern_lower);
        best_window_score = best_window_score.max(score);

        // Early exit on perfect match
        if best_window_score >= 1.0 {
            return 1.0;
        }
    }

    // Return the best of full-string and sliding-window scores
    full_score.max(best_window_score)
}

/// Main matching function: find the best matching error entry for the given parsed error.
pub fn find_match(
    parsed: &ParsedError,
    entries: &[ErrorEntry],
    fuzzy_threshold: f64,
) -> RemapResult {
    let original_code = parsed.code.clone().unwrap_or_default();
    let original_message = parsed.message.clone().unwrap_or_default();

    // Step 1: Exact match by code → key
    let code_matches: Vec<&ErrorEntry> = entries
        .iter()
        .filter(|e| e.key == original_code)
        .collect();

    log::info!(
        "Code '{}': found {} exact key matches",
        original_code,
        code_matches.len()
    );

    // If exactly one match by code — use it
    if code_matches.len() == 1 {
        let entry = code_matches[0];
        let desc = entry
            .custom_desc
            .clone()
            .unwrap_or_else(|| original_message.clone());
        log::info!("Exact single match: code={}, desc={}", entry.code, desc);
        return RemapResult {
            code: entry.code.clone(),
            custom_desc: desc,
            matched: true,
        };
    }

    // Step 2: Fuzzy matching
    let search_pool: Vec<&ErrorEntry> = if code_matches.is_empty() {
        // No matches by code — search the entire dictionary
        log::info!("No code matches, searching entire dictionary");
        entries.iter().collect()
    } else {
        // Multiple matches by code — narrow down by fuzzy text matching
        log::info!(
            "Multiple code matches ({}), narrowing by text",
            code_matches.len()
        );
        code_matches
    };

    if original_message.is_empty() {
        log::warn!("No message text available for fuzzy matching");
        return RemapResult {
            code: original_code,
            custom_desc: original_message,
            matched: false,
        };
    }

    // Score each candidate
    let mut scored: Vec<ScoredEntry> = search_pool
        .iter()
        .map(|entry| {
            let score = fuzzy_score(&original_message, &entry.substring);
            log::debug!(
                "  candidate key={} substring='{}' → score={:.3}",
                entry.key,
                entry.substring,
                score
            );
            ScoredEntry { entry, score }
        })
        .filter(|s| s.score >= fuzzy_threshold)
        .collect();

    // Sort by score descending
    scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

    if let Some(best) = scored.first() {
        let entry = best.entry;
        let desc = entry
            .custom_desc
            .clone()
            .unwrap_or_else(|| original_message.clone());
        log::info!(
            "Best fuzzy match: key={} code={} score={:.3} desc={}",
            entry.key,
            entry.code,
            best.score,
            desc
        );
        RemapResult {
            code: entry.code.clone(),
            custom_desc: desc,
            matched: true,
        }
    } else {
        log::info!("No match found above threshold {}", fuzzy_threshold);
        RemapResult {
            code: original_code,
            custom_desc: original_message,
            matched: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_entries() -> Vec<ErrorEntry> {
        vec![
            ErrorEntry {
                key: "2001".into(),
                substring: "unexpected symbol:".into(),
                custom_desc: Some("Недопустимый символ в назначении перевода".into()),
                code: "81002".into(),
            },
            ErrorEntry {
                key: "2002".into(),
                substring: "Уточните у получателя".into(),
                custom_desc: None,
                code: "81001".into(),
            },
            ErrorEntry {
                key: "3011".into(),
                substring: "Система быстрых платежей заблокировала".into(),
                custom_desc: None,
                code: "81004".into(),
            },
            ErrorEntry {
                key: "3011".into(),
                substring: "Не пройден фрод".into(),
                custom_desc: Some("Перевод отклонён банком получателя".into()),
                code: "81005".into(),
            },
            ErrorEntry {
                key: "3012".into(),
                substring: "отклонен провайдером".into(),
                custom_desc: Some("Перевод отклонён банком получателя".into()),
                code: "81006".into(),
            },
        ]
    }

    #[test]
    fn test_exact_single_code_match() {
        let entries = sample_entries();
        let parsed = ParsedError {
            code: Some("2002".into()),
            message: Some("Уточните у получателя реквизиты".into()),
        };
        let result = find_match(&parsed, &entries, 0.4);
        assert!(result.matched);
        assert_eq!(result.code, "81001");
        // No customDesc in YAML → use original message
        assert_eq!(result.custom_desc, "Уточните у получателя реквизиты");
    }

    #[test]
    fn test_exact_single_code_match_with_custom_desc() {
        let entries = sample_entries();
        let parsed = ParsedError {
            code: Some("2001".into()),
            message: Some("Got unexpected symbol: @ in input".into()),
        };
        let result = find_match(&parsed, &entries, 0.4);
        assert!(result.matched);
        assert_eq!(result.code, "81002");
        assert_eq!(result.custom_desc, "Недопустимый символ в назначении перевода");
    }

    #[test]
    fn test_multiple_code_matches_fuzzy_narrows() {
        let entries = sample_entries();
        // key "3011" matches two entries — fuzzy should pick the right one
        let parsed = ParsedError {
            code: Some("3011".into()),
            message: Some("Не пройден фрод-мониторинг операции".into()),
        };
        let result = find_match(&parsed, &entries, 0.4);
        assert!(result.matched);
        assert_eq!(result.code, "81005");
        assert_eq!(result.custom_desc, "Перевод отклонён банком получателя");
    }

    #[test]
    fn test_no_code_match_fuzzy_on_all() {
        let entries = sample_entries();
        let parsed = ParsedError {
            code: Some("9999".into()),
            message: Some("Перевод отклонен провайдером получателя".into()),
        };
        let result = find_match(&parsed, &entries, 0.4);
        assert!(result.matched);
        assert_eq!(result.code, "81006");
        assert_eq!(result.custom_desc, "Перевод отклонён банком получателя");
    }

    #[test]
    fn test_no_match_at_all() {
        let entries = sample_entries();
        let parsed = ParsedError {
            code: Some("9999".into()),
            message: Some("Completely unrelated error text".into()),
        };
        let result = find_match(&parsed, &entries, 0.4);
        assert!(!result.matched);
        assert_eq!(result.code, "9999");
        assert_eq!(result.custom_desc, "Completely unrelated error text");
    }

    #[test]
    fn test_fuzzy_score_exact_substring() {
        let score = fuzzy_score("Система быстрых платежей заблокировала перевод", "Система быстрых платежей заблокировала");
        assert_eq!(score, 1.0);
    }

    #[test]
    fn test_fuzzy_score_partial_match() {
        let score = fuzzy_score("Не пройден фрод-мониторинг операции", "Не пройден фрод");
        assert_eq!(score, 1.0); // substring is contained
    }

    #[test]
    fn test_fuzzy_score_no_match() {
        let score = fuzzy_score("Hello world", "Абсолютно другой текст");
        assert!(score < 0.3);
    }
}
