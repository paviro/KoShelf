//! Generic comma-separated token set used by include-style query parameters.

use std::collections::HashSet;
use std::hash::Hash;

use crate::server::api::responses::error::ApiErrorCode;

/// Trait implemented by each token enum to plug into [`TokenSet`].
pub trait SetToken: Copy + Eq + Hash + 'static {
    /// Parse a single token string. Returns `None` for unknown values.
    fn parse_token(value: &str) -> Option<Self>;

    /// Human-readable list of valid tokens for error messages.
    fn valid_tokens() -> &'static str;

    /// If this token set supports an `"all"` shortcut, return every variant.
    /// Default: `None` (no `"all"` support).
    fn all_variants() -> Option<&'static [Self]> {
        None
    }
}

/// A validated set of include tokens parsed from a comma-separated string.
#[derive(Debug, Clone)]
pub struct TokenSet<T: SetToken> {
    tokens: HashSet<T>,
}

impl<T: SetToken> Default for TokenSet<T> {
    fn default() -> Self {
        Self {
            tokens: HashSet::new(),
        }
    }
}

impl<T: SetToken> TokenSet<T> {
    /// Parse a comma-separated include string with strict validation.
    ///
    /// - Duplicates are silently ignored.
    /// - Empty / whitespace-only segments are skipped.
    /// - If the token set supports `"all"`, encountering it returns every variant.
    /// - Unknown tokens produce an error listing valid options.
    pub fn parse(value: Option<&str>) -> Result<Self, (ApiErrorCode, String)> {
        let Some(value) = value else {
            return Ok(Self::default());
        };

        let value = value.trim();
        if value.is_empty() {
            return Ok(Self::default());
        }

        let mut tokens = HashSet::new();

        for raw in value.split(',') {
            let token = raw.trim();
            if token.is_empty() {
                continue;
            }
            if token == "all"
                && let Some(all) = T::all_variants()
            {
                return Ok(Self {
                    tokens: all.iter().copied().collect(),
                });
            }
            match T::parse_token(token) {
                Some(t) => {
                    tokens.insert(t);
                }
                None => {
                    return Err((
                        ApiErrorCode::InvalidQuery,
                        format!(
                            "unknown include token '{token}'; valid tokens are: {}",
                            T::valid_tokens(),
                        ),
                    ));
                }
            }
        }

        Ok(Self { tokens })
    }

    /// Create a set containing all variants (requires `all_variants()` support).
    pub fn all() -> Self
    where
        T: SetToken<>,
    {
        Self {
            tokens: T::all_variants()
                .expect("all() requires SetToken::all_variants()")
                .iter()
                .copied()
                .collect(),
        }
    }

    pub fn has(&self, token: T) -> bool {
        self.tokens.contains(&token)
    }
}
