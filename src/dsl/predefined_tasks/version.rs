//! Semantic Versioning Support for Predefined Tasks
//!
//! This module provides semantic versioning constraint parsing and matching
//! following the semver specification.

use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors that can occur during version parsing or matching
#[derive(Debug, Error)]
pub enum VersionError {
    /// Invalid version string
    #[error("Invalid version '{0}': {1}")]
    InvalidVersion(String, String),

    /// Invalid version constraint
    #[error("Invalid version constraint '{0}': {1}")]
    InvalidConstraint(String, String),

    /// Version does not match constraint
    #[error("Version {version} does not match constraint {constraint}")]
    VersionMismatch { version: String, constraint: String },
}

/// Version constraint for task dependencies
///
/// Supports standard semver constraints:
/// - Exact: `1.2.3`
/// - Caret: `^1.2.3` (>=1.2.3 <2.0.0)
/// - Tilde: `~1.2.3` (>=1.2.3 <1.3.0)
/// - Wildcard: `1.x`, `1.2.x`
/// - Range: `>=1.2.0 <2.0.0`
/// - Latest: `latest` (special case)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionConstraint {
    /// Original constraint string
    constraint: String,

    /// Parsed semver requirement (None for "latest")
    #[serde(skip)]
    req: Option<VersionReq>,

    /// Whether this is a "latest" constraint
    is_latest: bool,
}

impl VersionConstraint {
    /// Parse a version constraint string
    ///
    /// # Examples
    ///
    /// ```
    /// use periplon_sdk::dsl::predefined_tasks::VersionConstraint;
    ///
    /// let constraint = VersionConstraint::parse("^1.2.0").unwrap();
    /// assert!(constraint.matches("1.2.3").unwrap());
    /// assert!(constraint.matches("1.9.0").unwrap());
    /// assert!(!constraint.matches("2.0.0").unwrap());
    /// ```
    pub fn parse(s: &str) -> Result<Self, VersionError> {
        let trimmed = s.trim();

        // Handle "latest" special case
        if trimmed.eq_ignore_ascii_case("latest") {
            return Ok(VersionConstraint {
                constraint: "latest".to_string(),
                req: None,
                is_latest: true,
            });
        }

        // Parse as semver requirement
        let req = VersionReq::parse(trimmed)
            .map_err(|e| VersionError::InvalidConstraint(trimmed.to_string(), e.to_string()))?;

        Ok(VersionConstraint {
            constraint: trimmed.to_string(),
            req: Some(req),
            is_latest: false,
        })
    }

    /// Check if a version string matches this constraint
    pub fn matches(&self, version: &str) -> Result<bool, VersionError> {
        // "latest" matches any version
        if self.is_latest {
            return Ok(true);
        }

        let version = Version::parse(version)
            .map_err(|e| VersionError::InvalidVersion(version.to_string(), e.to_string()))?;

        Ok(self
            .req
            .as_ref()
            .map(|req| req.matches(&version))
            .unwrap_or(false))
    }

    /// Get the constraint string
    pub fn as_str(&self) -> &str {
        &self.constraint
    }

    /// Check if this is a "latest" constraint
    pub fn is_latest(&self) -> bool {
        self.is_latest
    }

    /// Get the underlying VersionReq
    pub fn requirement(&self) -> Option<&VersionReq> {
        self.req.as_ref()
    }
}

impl std::fmt::Display for VersionConstraint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.constraint)
    }
}

impl std::str::FromStr for VersionConstraint {
    type Err = VersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        VersionConstraint::parse(s)
    }
}

/// Find the best matching version from a list of available versions
///
/// Returns the highest version that matches the constraint.
pub fn find_best_match(
    constraint: &VersionConstraint,
    available_versions: &[String],
) -> Result<Option<String>, VersionError> {
    if constraint.is_latest() {
        // For "latest", return the highest version
        return find_highest_version(available_versions);
    }

    let mut matching_versions: Vec<Version> = Vec::new();

    for version_str in available_versions {
        let version = Version::parse(version_str)
            .map_err(|e| VersionError::InvalidVersion(version_str.to_string(), e.to_string()))?;

        if constraint.matches(version_str)? {
            matching_versions.push(version);
        }
    }

    if matching_versions.is_empty() {
        return Ok(None);
    }

    // Sort and return the highest matching version
    matching_versions.sort();
    Ok(matching_versions.last().map(|v| v.to_string()))
}

/// Find the highest version from a list
fn find_highest_version(versions: &[String]) -> Result<Option<String>, VersionError> {
    let mut parsed_versions: Vec<Version> = Vec::new();

    for version_str in versions {
        let version = Version::parse(version_str)
            .map_err(|e| VersionError::InvalidVersion(version_str.to_string(), e.to_string()))?;
        parsed_versions.push(version);
    }

    if parsed_versions.is_empty() {
        return Ok(None);
    }

    parsed_versions.sort();
    Ok(parsed_versions.last().map(|v| v.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_exact_version() {
        // Note: semver requires "=1.2.3" for exact matching
        let constraint = VersionConstraint::parse("=1.2.3").unwrap();
        assert!(constraint.matches("1.2.3").unwrap());
        assert!(!constraint.matches("1.2.4").unwrap());
        assert!(!constraint.matches("1.3.0").unwrap());
        assert!(!constraint.matches("2.0.0").unwrap());

        // Test bare version (treated as caret: ^1.2.3 = >=1.2.3, <2.0.0)
        let constraint2 = VersionConstraint::parse("1.2.3").unwrap();
        assert!(constraint2.matches("1.2.3").unwrap());
        assert!(constraint2.matches("1.2.4").unwrap()); // Caret allows patches
        assert!(constraint2.matches("1.3.0").unwrap()); // And minor bumps
        assert!(!constraint2.matches("2.0.0").unwrap()); // But not major bumps
    }

    #[test]
    fn test_parse_caret_constraint() {
        let constraint = VersionConstraint::parse("^1.2.3").unwrap();
        assert!(constraint.matches("1.2.3").unwrap());
        assert!(constraint.matches("1.2.4").unwrap());
        assert!(constraint.matches("1.9.0").unwrap());
        assert!(!constraint.matches("2.0.0").unwrap());
        assert!(!constraint.matches("1.1.0").unwrap());
    }

    #[test]
    fn test_parse_tilde_constraint() {
        let constraint = VersionConstraint::parse("~1.2.3").unwrap();
        assert!(constraint.matches("1.2.3").unwrap());
        assert!(constraint.matches("1.2.4").unwrap());
        assert!(!constraint.matches("1.3.0").unwrap());
        assert!(!constraint.matches("2.0.0").unwrap());
    }

    #[test]
    fn test_parse_wildcard() {
        let constraint = VersionConstraint::parse("1.2.*").unwrap();
        assert!(constraint.matches("1.2.0").unwrap());
        assert!(constraint.matches("1.2.99").unwrap());
        assert!(!constraint.matches("1.3.0").unwrap());
    }

    #[test]
    fn test_parse_latest() {
        let constraint = VersionConstraint::parse("latest").unwrap();
        assert!(constraint.is_latest());
        assert!(constraint.matches("1.0.0").unwrap());
        assert!(constraint.matches("999.999.999").unwrap());
    }

    #[test]
    fn test_parse_range() {
        let constraint = VersionConstraint::parse(">=1.2.0, <2.0.0").unwrap();
        assert!(constraint.matches("1.2.0").unwrap());
        assert!(constraint.matches("1.9.9").unwrap());
        assert!(!constraint.matches("2.0.0").unwrap());
        assert!(!constraint.matches("1.1.9").unwrap());
    }

    #[test]
    fn test_find_best_match() {
        let constraint = VersionConstraint::parse("^1.2.0").unwrap();
        let versions = vec![
            "1.0.0".to_string(),
            "1.2.0".to_string(),
            "1.2.5".to_string(),
            "1.9.0".to_string(),
            "2.0.0".to_string(),
        ];

        let best = find_best_match(&constraint, &versions).unwrap();
        assert_eq!(best, Some("1.9.0".to_string()));
    }

    #[test]
    fn test_find_best_match_latest() {
        let constraint = VersionConstraint::parse("latest").unwrap();
        let versions = vec![
            "1.0.0".to_string(),
            "1.2.0".to_string(),
            "2.0.0".to_string(),
            "1.9.0".to_string(),
        ];

        let best = find_best_match(&constraint, &versions).unwrap();
        assert_eq!(best, Some("2.0.0".to_string()));
    }

    #[test]
    fn test_find_best_match_no_match() {
        let constraint = VersionConstraint::parse("^3.0.0").unwrap();
        let versions = vec!["1.0.0".to_string(), "2.0.0".to_string()];

        let best = find_best_match(&constraint, &versions).unwrap();
        assert_eq!(best, None);
    }

    #[test]
    fn test_invalid_version() {
        let constraint = VersionConstraint::parse("^1.2.0").unwrap();
        let result = constraint.matches("not-a-version");
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_constraint() {
        let result = VersionConstraint::parse("invalid constraint");
        assert!(result.is_err());
    }
}
