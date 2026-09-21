//! Search ranking: how well a query matches a name, plus a bonus for things opened often
//! and recently ("frecency"). Text relevance dominates: an exact name match always beats a
//! merely frequent partial match, but among similar matches the one you use wins.

use crate::models::usage::Usage;

const EXACT: f64 = 100.0;
const PREFIX: f64 = 80.0;
const WORD_START: f64 = 65.0;
const SUBSTRING: f64 = 50.0;
const FUZZY_BASE: f64 = 20.0;
const FUZZY_MAX_BONUS: f64 = 10.0;
const SECONDARY: f64 = 15.0;

/// Highest bonus frecency can add (below the gap between "substring" and "exact").
const MAX_FRECENCY: f64 = 25.0;

fn is_boundary(c: char) -> bool {
    !c.is_alphanumeric()
}

/// Do the characters of `needle` appear in `haystack` in order? Returns how tightly: 1.0 when
/// they are contiguous, approaching 0 as they spread out.
fn subsequence_tightness(needle: &str, haystack: &str) -> Option<f64> {
    let mut hay = haystack.char_indices();
    let mut first = None;
    let mut last = 0;
    let mut matched = 0;
    for n in needle.chars() {
        let (i, _) = hay.find(|(_, h)| *h == n)?;
        first.get_or_insert(i);
        last = i;
        matched += 1;
    }
    let span = last - first? + 1;
    Some(matched as f64 / span.max(matched) as f64)
}

fn token_score(token: &str, primary: &str, secondary: Option<&str>) -> Option<f64> {
    if primary == token {
        return Some(EXACT);
    }
    if primary.starts_with(token) {
        return Some(PREFIX);
    }
    // Occurrences are checked all the way along: "docs" in "my-docs-docs" starts a word
    // even if an earlier hit sits mid-word.
    let mut mid_word = false;
    for (pos, _) in primary.match_indices(token) {
        if primary[..pos].chars().next_back().is_some_and(is_boundary) {
            return Some(WORD_START);
        }
        mid_word = true;
    }
    if mid_word {
        return Some(SUBSTRING);
    }
    if token.chars().count() >= 2 {
        if let Some(t) = subsequence_tightness(token, primary) {
            return Some(FUZZY_BASE + FUZZY_MAX_BONUS * t);
        }
    }
    if secondary.is_some_and(|s| s.contains(token)) {
        return Some(SECONDARY);
    }
    None
}

/// Relevance of `query` to a `primary` name (and optionally a `secondary` string such as a
/// path). Every space-separated word of the query has to match; `None` if any doesn't.
pub fn text_score(query: &str, primary: &str, secondary: Option<&str>) -> Option<f64> {
    let primary = primary.to_lowercase();
    let secondary = secondary.map(str::to_lowercase);
    let mut total = 0.0;
    let mut words = 0;
    for token in query.to_lowercase().split_whitespace() {
        total += token_score(token, &primary, secondary.as_deref())?;
        words += 1;
    }
    (words > 0).then(|| total / words as f64)
}

/// Bonus for how often and how recently something was opened.
pub fn frecency_bonus(usage: Option<&Usage>, now: f64) -> f64 {
    let Some(u) = usage else { return 0.0 };
    let age_days = ((now - u.last) / 86_400.0).max(0.0);
    let decay = match age_days {
        d if d <= 4.0 => 1.0,
        d if d <= 14.0 => 0.7,
        d if d <= 60.0 => 0.4,
        _ => 0.2,
    };
    (8.0 * (u.count as f64).ln_1p() * decay).min(MAX_FRECENCY)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(q: &str, name: &str) -> f64 {
        text_score(q, name, None).unwrap_or(-1.0)
    }

    #[test]
    fn relevance_tiers_are_ordered() {
        let exact = s("vori", "vori");
        let prefix = s("vori", "vori-docs");
        let word = s("docs", "vori-docs");
        let substring = s("ori", "vori-docs");
        let fuzzy = s("vdc", "vori-docs");
        assert!(exact > prefix && prefix > word && word > substring && substring > fuzzy);
        assert!(fuzzy > 0.0);
        assert_eq!(text_score("zzz", "vori-docs", None), None);
    }

    #[test]
    fn a_word_start_later_in_the_name_still_counts_as_a_word_start() {
        // first "docs" is mid-word ("xdocs"), the second starts a word
        assert_eq!(s("docs", "xdocs-docs"), WORD_START);
        assert_eq!(s("docs", "xdocs"), SUBSTRING);
    }

    #[test]
    fn matching_is_case_insensitive() {
        assert_eq!(s("VORI", "vori"), s("vori", "VORI"));
        assert_eq!(s("Vori", "vori"), 100.0);
    }

    #[test]
    fn a_tighter_fuzzy_match_scores_higher() {
        assert!(s("abc", "a-b-c") > s("abc", "a-----b-----c"));
    }

    #[test]
    fn every_word_of_the_query_must_match() {
        assert!(text_score("api gate", "api-gateway", None).is_some());
        assert_eq!(text_score("api zzz", "api-gateway", None), None);
        assert_eq!(text_score("   ", "api-gateway", None), None);
    }

    #[test]
    fn a_path_only_match_ranks_below_any_name_match() {
        let by_path = text_score("workspace", "api", Some("/home/u/workspace/api")).unwrap();
        assert_eq!(by_path, SECONDARY);
        assert!(by_path < s("ap", "api"));
    }

    #[test]
    fn single_letters_do_not_fuzzy_match_everything() {
        // a lone character only matches as a plain substring, never as a scattered subsequence
        assert!(text_score("x", "api", None).is_none());
    }

    #[test]
    fn frecency_grows_with_use_and_fades_with_age() {
        let now = 1_000_000_000.0;
        let day = 86_400.0;
        let fresh = Usage { count: 5, last: now - day };
        let old = Usage { count: 5, last: now - 90.0 * day };
        let once = Usage { count: 1, last: now - day };
        assert!(frecency_bonus(Some(&fresh), now) > frecency_bonus(Some(&old), now));
        assert!(frecency_bonus(Some(&fresh), now) > frecency_bonus(Some(&once), now));
        assert_eq!(frecency_bonus(None, now), 0.0);
        let huge = Usage { count: 1_000_000, last: now };
        assert_eq!(frecency_bonus(Some(&huge), now), MAX_FRECENCY);
    }

    #[test]
    fn frequent_use_reorders_similar_matches_but_never_beats_an_exact_one() {
        let now = 1_000_000_000.0;
        let heavy = Usage { count: 50, last: now };
        let total = |q, name, u: Option<&Usage>| s(q, name) + frecency_bonus(u, now);
        // two substring matches: the one you open all the time wins
        assert!(total("api", "my-api", Some(&heavy)) > total("api", "old-api", None));
        // but an exact name match stays on top of a frequent partial one
        assert!(total("api", "api", None) > total("api", "my-api", Some(&heavy)));
    }
}
