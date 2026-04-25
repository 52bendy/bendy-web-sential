use std::path::MAIN_SEPARATOR_STR;

pub fn validate_input<T: AsRef<str>>(value: T) -> bool {
    let s = value.as_ref();
    // Path traversal
    if s.contains("..") || s.contains(MAIN_SEPARATOR_STR) && MAIN_SEPARATOR_STR != "/" {
        return false;
    }
    // Null bytes
    if s.contains('\0') {
        return false;
    }
    // SQL injection patterns
    let sql_patterns = [
        "';", "--", "/*", "*/", "xp_", "sp_", "exec", "execute",
        "union", "select", "insert", "delete", "update", "drop",
    ];
    let lower = s.to_lowercase();
    for pattern in sql_patterns {
        if lower.contains(pattern) {
            // Allow in normal text fields, block in query params
            return false;
        }
    }
    // XSS patterns
    let xss_patterns = ["<script", "javascript:", "onerror=", "onload=", "onclick="];
    let lower = s.to_lowercase();
    for pattern in xss_patterns {
        if lower.contains(pattern) {
            return false;
        }
    }
    true
}

pub fn sanitize_string(input: &str) -> String {
    input
        .chars()
        .filter(|&c| !c.is_control() || c == '\n' || c == '\r' || c == '\t')
        .filter(|&c| c != '\0')
        .collect()
}

pub fn validate_domain(domain: &str) -> bool {
    if domain.is_empty() || domain.len() > 253 {
        return false;
    }
    if domain.starts_with('.') || domain.ends_with('.') {
        return false;
    }
    // Basic hostname validation
    domain.split('.').all(|label| {
        !label.is_empty()
        && label.len() <= 63
        && label.chars().all(|c| c.is_alphanumeric() || c == '-')
        && !label.starts_with('-')
        && !label.ends_with('-')
    })
}

pub fn validate_path_pattern(pattern: &str) -> bool {
    if pattern.is_empty() || pattern.len() > 256 {
        return false;
    }
    // Allow /, *, and alphanumeric plus common URL chars
    pattern.chars().all(|c| {
        c.is_alphanumeric() || c == '/' || c == '*' || c == '-' || c == '_' || c == '.' || c == '?'
    })
}

pub fn validate_url(url: &str) -> bool {
    if url.is_empty() || url.len() > 2048 {
        return false;
    }
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return false;
    }
    true
}
