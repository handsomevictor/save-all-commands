use crate::store::{Command, Store};
use nucleo_matcher::{
    pattern::{AtomKind, CaseMatching, Normalization, Pattern},
    Config, Matcher, Utf32Str,
};

pub struct SearchResult {
    pub command: Command,
    pub score: u32,
    pub folder_path: Vec<String>,
}

pub struct Searcher {
    matcher: Matcher,
}

impl Searcher {
    pub fn new() -> Self {
        Self {
            matcher: Matcher::new(Config::DEFAULT),
        }
    }

    /// Fuzzy search across all commands in store.
    ///
    /// Ranking:
    ///   1st priority: cmd field exactly contains query (case-insensitive)
    ///   2nd priority: desc field exactly contains query (case-insensitive)
    ///   3rd priority: weighted fuzzy score: cmd*3, desc*2, comment*1, tags*1
    ///   4th priority (tie): last_used closer to now ranks higher
    ///   5th priority (never used): lower command id first
    pub fn fuzzy_search(&mut self, query: &str, store: &Store) -> Vec<SearchResult> {
        if query.is_empty() {
            // Return all commands ordered by id
            let mut results: Vec<SearchResult> = store
                .commands
                .iter()
                .map(|cmd| SearchResult {
                    command: cmd.clone(),
                    score: 0,
                    folder_path: store.breadcrumb(&cmd.folder),
                })
                .collect();
            results.sort_by_key(|r| r.command.id);
            return results;
        }

        let query_lower = query.to_lowercase();

        let pattern = Pattern::new(
            query,
            CaseMatching::Ignore,
            Normalization::Smart,
            AtomKind::Fuzzy,
        );

        let mut results: Vec<SearchResult> = store
            .commands
            .iter()
            .filter_map(|cmd| {
                // Priority 1: cmd contains query exactly (case-insensitive)
                let cmd_exact = cmd.cmd.to_lowercase().contains(&query_lower);
                // Priority 2: desc contains query exactly (case-insensitive)
                let desc_exact = cmd.desc.to_lowercase().contains(&query_lower);

                // Priority 3: weighted fuzzy score
                let fuzzy_score = self.weighted_fuzzy_score(cmd, &pattern);

                // Only include if there's any match signal
                if !cmd_exact && !desc_exact && fuzzy_score == 0 {
                    return None;
                }

                // Encode priority into score:
                // Use a large base to separate priority tiers
                // Tier 1 (cmd exact): score >= 3_000_000
                // Tier 2 (desc exact): score >= 2_000_000
                // Tier 3 (fuzzy only): score = fuzzy_score (up to ~1_000_000)
                let base_score = if cmd_exact {
                    3_000_000u32 + fuzzy_score
                } else if desc_exact {
                    2_000_000u32 + fuzzy_score
                } else {
                    fuzzy_score
                };

                Some(SearchResult {
                    command: cmd.clone(),
                    score: base_score,
                    folder_path: store.breadcrumb(&cmd.folder),
                })
            })
            .collect();

        // Sort: higher score first; tie-break by last_used (more recent first), then by id (lower first)
        results.sort_by(|a, b| {
            b.score
                .cmp(&a.score)
                .then_with(|| {
                    let a_ts = parse_last_used(&a.command.last_used);
                    let b_ts = parse_last_used(&b.command.last_used);
                    b_ts.cmp(&a_ts) // more recent (larger timestamp) first
                })
                .then_with(|| a.command.id.cmp(&b.command.id)) // lower id first
        });

        results
    }

    /// Exact search: returns commands whose haystack contains query as substring (case-insensitive).
    /// haystack = "{cmd} {desc} {comment} {tags joined by space}"
    pub fn exact_search(&self, query: &str, store: &Store) -> Vec<SearchResult> {
        if query.is_empty() {
            // Return all commands ordered by id
            let mut results: Vec<SearchResult> = store
                .commands
                .iter()
                .map(|cmd| SearchResult {
                    command: cmd.clone(),
                    score: 0,
                    folder_path: store.breadcrumb(&cmd.folder),
                })
                .collect();
            results.sort_by_key(|r| r.command.id);
            return results;
        }

        let query_lower = query.to_lowercase();

        let mut results: Vec<SearchResult> = store
            .commands
            .iter()
            .filter_map(|cmd| {
                let haystack = format!(
                    "{} {} {} {}",
                    cmd.cmd,
                    cmd.desc,
                    cmd.comment,
                    cmd.tags.join(" ")
                );
                if haystack.to_lowercase().contains(&query_lower) {
                    Some(SearchResult {
                        command: cmd.clone(),
                        score: 0,
                        folder_path: store.breadcrumb(&cmd.folder),
                    })
                } else {
                    None
                }
            })
            .collect();

        results.sort_by_key(|r| r.command.id);
        results
    }

    fn weighted_fuzzy_score(&mut self, cmd: &Command, pattern: &Pattern) -> u32 {
        let cmd_score = fuzzy_score_field(&mut self.matcher, pattern, &cmd.cmd);
        let desc_score = fuzzy_score_field(&mut self.matcher, pattern, &cmd.desc);
        let comment_score = fuzzy_score_field(&mut self.matcher, pattern, &cmd.comment);
        let tags_text = cmd.tags.join(" ");
        let tags_score = fuzzy_score_field(&mut self.matcher, pattern, &tags_text);

        cmd_score * 3 + desc_score * 2 + comment_score + tags_score
    }
}

fn fuzzy_score_field(matcher: &mut Matcher, pattern: &Pattern, text: &str) -> u32 {
    if text.is_empty() {
        return 0;
    }
    let mut buf = Vec::new();
    let haystack = Utf32Str::new(text, &mut buf);
    pattern
        .score(haystack, matcher)
        .unwrap_or(0)
}

/// Parse a last_used string into an i64 timestamp (seconds since epoch).
/// Returns i64::MIN for empty/invalid strings (never used = lowest priority).
fn parse_last_used(last_used: &str) -> i64 {
    if last_used.is_empty() {
        return i64::MIN;
    }
    // Try to parse as various ISO 8601 formats
    use chrono::{DateTime, NaiveDate, NaiveDateTime};

    // Try full RFC3339/ISO8601 with timezone
    if let Ok(dt) = DateTime::parse_from_rfc3339(last_used) {
        return dt.timestamp();
    }
    // Try naive datetime
    if let Ok(dt) = NaiveDateTime::parse_from_str(last_used, "%Y-%m-%dT%H:%M:%S") {
        return dt.and_utc().timestamp();
    }
    if let Ok(dt) = NaiveDateTime::parse_from_str(last_used, "%Y-%m-%d %H:%M:%S") {
        return dt.and_utc().timestamp();
    }
    // Try date only
    if let Ok(d) = NaiveDate::parse_from_str(last_used, "%Y-%m-%d") {
        return d.and_hms_opt(0, 0, 0).map(|dt| dt.and_utc().timestamp()).unwrap_or(i64::MIN);
    }
    i64::MIN
}

impl Default for Searcher {
    fn default() -> Self {
        Self::new()
    }
}
