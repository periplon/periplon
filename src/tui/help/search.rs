//! Help search engine
//!
//! Full-text search across help content with:
//! - Keyword matching
//! - Fuzzy search
//! - Relevance ranking
//! - Search suggestions

use super::content::{HelpContent, HelpTopic};
use std::collections::HashMap;

/// Search result with relevance score
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// Topic that matched
    pub topic: HelpTopic,
    /// Relevance score (0.0 - 1.0, higher is more relevant)
    pub score: f32,
    /// Matched keywords
    pub matched_keywords: Vec<String>,
    /// Excerpt from content showing match context
    pub excerpt: String,
}

/// Help search engine with full-text indexing
#[derive(Debug)]
pub struct HelpSearchEngine {
    /// Reference to help content
    content: HelpContent,
    /// Inverted index: keyword -> topic IDs
    index: HashMap<String, Vec<String>>,
}

impl HelpSearchEngine {
    /// Create new search engine
    pub fn new(content: HelpContent) -> Self {
        let mut engine = Self {
            content,
            index: HashMap::new(),
        };
        engine.build_index();
        engine
    }

    /// Build inverted index for fast searching
    fn build_index(&mut self) {
        for topic in self.content.all_topics() {
            // Index topic title
            for word in Self::tokenize(&topic.title) {
                self.index
                    .entry(word.to_lowercase())
                    .or_default()
                    .push(topic.id.clone());
            }

            // Index topic content
            for word in Self::tokenize(&topic.content) {
                self.index
                    .entry(word.to_lowercase())
                    .or_default()
                    .push(topic.id.clone());
            }

            // Index keywords
            for keyword in &topic.keywords {
                self.index
                    .entry(keyword.to_lowercase())
                    .or_default()
                    .push(topic.id.clone());
            }
        }
    }

    /// Tokenize text into searchable words
    fn tokenize(text: &str) -> Vec<String> {
        text.split(|c: char| !c.is_alphanumeric() && c != '_')
            .filter(|s| !s.is_empty() && s.len() > 2) // Ignore very short words
            .map(|s| s.to_string())
            .collect()
    }

    /// Search for topics matching the query
    pub fn search(&self, query: &str) -> Vec<SearchResult> {
        if query.trim().is_empty() {
            return Vec::new();
        }

        let query_terms = Self::tokenize(&query.to_lowercase());
        let mut topic_scores: HashMap<String, (f32, Vec<String>)> = HashMap::new();

        for term in &query_terms {
            // Exact matches
            if let Some(topic_ids) = self.index.get(term) {
                for topic_id in topic_ids {
                    let entry = topic_scores.entry(topic_id.clone()).or_insert((0.0, Vec::new()));
                    entry.0 += 1.0; // Exact match gets full point
                    entry.1.push(term.clone());
                }
            }

            // Fuzzy matches (prefix matching)
            for (indexed_term, topic_ids) in &self.index {
                if indexed_term.starts_with(term) && indexed_term != term {
                    for topic_id in topic_ids {
                        let entry = topic_scores.entry(topic_id.clone()).or_insert((0.0, Vec::new()));
                        entry.0 += 0.5; // Prefix match gets half point
                        entry.1.push(indexed_term.clone());
                    }
                }
            }
        }

        // Convert to search results
        let mut results: Vec<SearchResult> = topic_scores
            .into_iter()
            .filter_map(|(topic_id, (score, matched_keywords))| {
                self.content.get_topic(&topic_id).map(|topic| {
                    let normalized_score = score / query_terms.len() as f32;
                    let excerpt = self.extract_excerpt(&topic.content, &query_terms);

                    SearchResult {
                        topic: topic.clone(),
                        score: normalized_score.min(1.0),
                        matched_keywords,
                        excerpt,
                    }
                })
            })
            .collect();

        // Sort by relevance (score descending)
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        results
    }

    /// Extract excerpt from content showing match context
    fn extract_excerpt(&self, content: &str, query_terms: &[String]) -> String {
        const EXCERPT_LENGTH: usize = 150;

        // Find first occurrence of any query term
        let content_lower = content.to_lowercase();
        let mut first_match_pos = None;

        for term in query_terms {
            if let Some(pos) = content_lower.find(term) {
                if first_match_pos.is_none() || pos < first_match_pos.unwrap() {
                    first_match_pos = Some(pos);
                }
            }
        }

        let start = if let Some(pos) = first_match_pos {
            // Start a bit before the match
            pos.saturating_sub(30)
        } else {
            0
        };

        let end = (start + EXCERPT_LENGTH).min(content.len());

        // Find word boundaries
        let excerpt_start = content[..start]
            .rfind(char::is_whitespace)
            .map(|p| p + 1)
            .unwrap_or(start);

        let excerpt_end = content[end..]
            .find(char::is_whitespace)
            .map(|p| end + p)
            .unwrap_or(end);

        let mut excerpt = content[excerpt_start..excerpt_end].trim().to_string();

        // Add ellipsis if truncated
        if excerpt_start > 0 {
            excerpt = format!("...{}", excerpt);
        }
        if excerpt_end < content.len() {
            excerpt = format!("{}...", excerpt);
        }

        excerpt
    }

    /// Get search suggestions based on partial input
    pub fn suggest(&self, partial: &str) -> Vec<String> {
        if partial.len() < 2 {
            return Vec::new();
        }

        let partial_lower = partial.to_lowercase();
        let mut suggestions: Vec<String> = self
            .index
            .keys()
            .filter(|k| k.starts_with(&partial_lower))
            .take(10)
            .map(|s| s.to_string())
            .collect();

        suggestions.sort();
        suggestions.dedup();
        suggestions
    }

    /// Get the underlying help content
    pub fn content(&self) -> &HelpContent {
        &self.content
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        let tokens = HelpSearchEngine::tokenize("Hello, world! This is a test.");
        assert!(tokens.contains(&"Hello".to_string()));
        assert!(tokens.contains(&"world".to_string()));
        assert!(tokens.contains(&"test".to_string()));
    }

    #[test]
    fn test_search() {
        let content = HelpContent::new();
        let engine = HelpSearchEngine::new(content);

        let results = engine.search("workflow");
        assert!(!results.is_empty());
    }

    #[test]
    fn test_search_case_insensitive() {
        let content = HelpContent::new();
        let engine = HelpSearchEngine::new(content);

        let results1 = engine.search("WORKFLOW");
        let results2 = engine.search("workflow");
        assert_eq!(results1.len(), results2.len());
    }

    #[test]
    fn test_suggest() {
        let content = HelpContent::new();
        let engine = HelpSearchEngine::new(content);

        let suggestions = engine.suggest("work");
        assert!(!suggestions.is_empty());
    }

    #[test]
    fn test_empty_query() {
        let content = HelpContent::new();
        let engine = HelpSearchEngine::new(content);

        let results = engine.search("");
        assert!(results.is_empty());
    }
}
