use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct RouteMatch {
    pub handler_key: String,
    pub parameters: HashMap<String, String>,
    pub is_match: bool,
}

impl RouteMatch {
    pub fn new(handler_key: String) -> Self {
        RouteMatch {
            handler_key,
            parameters: HashMap::new(),
            is_match: true,
        }
    }

    pub fn no_match() -> Self {
        RouteMatch {
            handler_key: String::new(),
            parameters: HashMap::new(),
            is_match: false,
        }
    }

    pub fn add_parameter(&mut self, key: &str, value: &str) {
        self.parameters.insert(key.to_string(), value.to_string());
    }
}

pub struct RouteMatcher {
    patterns: Vec<(String, String)>, // (pattern, handler_key)
}

impl RouteMatcher {
    pub fn new() -> Self {
        RouteMatcher {
            patterns: Vec::new(),
        }
    }

    /// Register a route pattern with its handler key
    pub fn register(&mut self, pattern: &str, handler_key: &str) {
        self.patterns.push((pattern.to_string(), handler_key.to_string()));
    }

    /// Match a request path against registered patterns
    pub fn match_path(&self, request_path: &str) -> RouteMatch {
        for (pattern, handler_key) in &self.patterns {
            if let Some(parameters) = Self::match_pattern(pattern, request_path) {
                let mut route_match = RouteMatch::new(handler_key.clone());
                route_match.parameters = parameters;
                return route_match;
            }
        }
        RouteMatch::no_match()
    }

    /// Match a single pattern against a request path
    fn match_pattern(pattern: &str, request_path: &str) -> Option<HashMap<String, String>> {
        let pattern_segments: Vec<&str> = pattern.split('/').filter(|s| !s.is_empty()).collect();
        let request_segments: Vec<&str> = request_path.split('/').filter(|s| !s.is_empty()).collect();

        if pattern_segments.len() != request_segments.len() {
            return None;
        }

        let mut parameters = HashMap::new();

        for (pattern_seg, request_seg) in pattern_segments.iter().zip(request_segments.iter()) {
            if pattern_seg.starts_with('[') && pattern_seg.ends_with(']') {
                // Dynamic parameter
                let param_name = &pattern_seg[1..pattern_seg.len() - 1];
                parameters.insert(param_name.to_string(), request_seg.to_string());
            } else if pattern_seg != request_seg {
                // Static segment doesn't match
                return None;
            }
        }

        Some(parameters)
    }

    /// Check if a pattern is valid
    pub fn is_valid_pattern(pattern: &str) -> bool {
        let segments: Vec<&str> = pattern.split('/').filter(|s| !s.is_empty()).collect();
        
        for segment in segments {
            if segment.starts_with('[') && segment.ends_with(']') {
                // Dynamic parameter - check it's not empty
                if segment.len() <= 2 {
                    return false;
                }
            } else if segment.contains('[') || segment.contains(']') {
                // Invalid brackets in static segment
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_static_pattern_matching() {
        let mut matcher = RouteMatcher::new();
        matcher.register("/users/profile", "user_profile");
        matcher.register("/posts", "posts_list");

        let match1 = matcher.match_path("/users/profile");
        assert!(match1.is_match);
        assert_eq!(match1.handler_key, "user_profile");
        assert!(match1.parameters.is_empty());

        let match2 = matcher.match_path("/posts");
        assert!(match2.is_match);
        assert_eq!(match2.handler_key, "posts_list");

        let no_match = matcher.match_path("/users/other");
        assert!(!no_match.is_match);
    }

    #[test]
    fn test_dynamic_pattern_matching() {
        let mut matcher = RouteMatcher::new();
        matcher.register("/users/[userId]", "user_detail");
        matcher.register("/posts/[postId]/comments/[commentId]", "comment_detail");

        let match1 = matcher.match_path("/users/123");
        assert!(match1.is_match);
        assert_eq!(match1.handler_key, "user_detail");
        assert_eq!(match1.parameters.get("userId"), Some(&"123".to_string()));

        let match2 = matcher.match_path("/posts/456/comments/789");
        assert!(match2.is_match);
        assert_eq!(match2.handler_key, "comment_detail");
        assert_eq!(match2.parameters.get("postId"), Some(&"456".to_string()));
        assert_eq!(match2.parameters.get("commentId"), Some(&"789".to_string()));
    }

    #[test]
    fn test_pattern_validation() {
        assert!(RouteMatcher::is_valid_pattern("/users/[userId]"));
        assert!(RouteMatcher::is_valid_pattern("/posts/[postId]/comments/[commentId]"));
        assert!(RouteMatcher::is_valid_pattern("/static/path"));
        
        assert!(!RouteMatcher::is_valid_pattern("/users/[]")); // Empty parameter
        assert!(!RouteMatcher::is_valid_pattern("/users/[userId")); // Unclosed bracket
        assert!(!RouteMatcher::is_valid_pattern("/users/[userId]/[postId")); // Unclosed bracket
    }
}
