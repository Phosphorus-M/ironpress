use crate::error::IronpressError;

/// Maximum allowed HTML input size (10 MB).
const MAX_INPUT_SIZE: usize = 10 * 1024 * 1024;

/// Maximum allowed nesting depth.
const MAX_NESTING_DEPTH: usize = 100;

/// Sanitize HTML input by removing dangerous elements and attributes.
pub fn sanitize_html(html: &str) -> Result<String, IronpressError> {
    // Check input size
    if html.len() > MAX_INPUT_SIZE {
        return Err(IronpressError::SecurityError(format!(
            "Input exceeds maximum size of {} bytes",
            MAX_INPUT_SIZE
        )));
    }

    // Check nesting depth
    if check_nesting_depth(html) > MAX_NESTING_DEPTH {
        return Err(IronpressError::SecurityError(
            "HTML nesting depth exceeds maximum".to_string(),
        ));
    }

    let mut result = html.to_string();

    // Remove script tags and content
    result = remove_tag_with_content(&result, "script");
    result = remove_tag_with_content(&result, "style");
    result = remove_tag_with_content(&result, "iframe");
    result = remove_tag_with_content(&result, "object");
    result = remove_tag_with_content(&result, "embed");
    result = remove_tag_with_content(&result, "form");

    // Remove event handler attributes
    result = remove_event_handlers(&result);

    // Remove javascript: URLs
    result = result.replace("javascript:", "");

    Ok(result)
}

fn remove_tag_with_content(html: &str, tag: &str) -> String {
    let mut result = html.to_string();
    let open = format!("<{tag}");
    let close = format!("</{tag}>");

    loop {
        let lower = result.to_ascii_lowercase();
        let start = lower.find(&open);
        let end = lower.find(&close);

        match (start, end) {
            (Some(s), Some(e)) => {
                let end_pos = e + close.len();
                result = format!("{}{}", &result[..s], &result[end_pos..]);
            }
            (Some(s), None) => {
                // Self-closing or unclosed — remove from start to end of tag
                if let Some(gt) = result[s..].find('>') {
                    result = format!("{}{}", &result[..s], &result[s + gt + 1..]);
                } else {
                    break;
                }
            }
            _ => break,
        }
    }

    result
}

fn remove_event_handlers(html: &str) -> String {
    // Only remove onXXX attributes inside HTML tags
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;

    let bytes = html.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        let c = bytes[i] as char;

        if c == '<' {
            in_tag = true;
            result.push(c);
            i += 1;
            continue;
        }

        if c == '>' {
            in_tag = false;
            result.push(c);
            i += 1;
            continue;
        }

        if in_tag && (c == 'o' || c == 'O') && i + 2 < bytes.len() {
            let next = bytes[i + 1] as char;
            if (next == 'n' || next == 'N') && (bytes[i + 2] as char).is_ascii_alphabetic() {
                // Check there's a space or start of tag before this
                let prev = if i > 0 { bytes[i - 1] as char } else { ' ' };
                if prev == ' ' || prev == '\t' || prev == '\n' {
                    // This looks like an event handler attribute — skip it
                    // Skip attribute name
                    let mut j = i;
                    while j < bytes.len()
                        && bytes[j] != b'='
                        && bytes[j] != b' '
                        && bytes[j] != b'>'
                    {
                        j += 1;
                    }
                    // Skip = and quoted value
                    if j < bytes.len() && bytes[j] == b'=' {
                        j += 1;
                        // Skip whitespace
                        while j < bytes.len() && (bytes[j] as char).is_whitespace() {
                            j += 1;
                        }
                        if j < bytes.len() && (bytes[j] == b'"' || bytes[j] == b'\'') {
                            let quote = bytes[j];
                            j += 1;
                            while j < bytes.len() && bytes[j] != quote {
                                j += 1;
                            }
                            if j < bytes.len() {
                                j += 1; // skip closing quote
                            }
                        } else {
                            // Unquoted — skip to space or >
                            while j < bytes.len() && bytes[j] != b' ' && bytes[j] != b'>' {
                                j += 1;
                            }
                        }
                    }
                    i = j;
                    continue;
                }
            }
        }

        result.push(c);
        i += 1;
    }

    result
}

fn check_nesting_depth(html: &str) -> usize {
    let mut depth: usize = 0;
    let mut max_depth: usize = 0;

    let mut in_tag = false;
    let mut is_closing = false;

    for c in html.chars() {
        match c {
            '<' => {
                in_tag = true;
                is_closing = false;
            }
            '/' if in_tag => {
                is_closing = true;
            }
            '>' if in_tag => {
                if is_closing {
                    depth = depth.saturating_sub(1);
                } else {
                    depth += 1;
                    max_depth = max_depth.max(depth);
                }
                in_tag = false;
            }
            _ => {}
        }
    }

    max_depth
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_script_tags() {
        let result =
            sanitize_html("<p>Hello</p><script>alert('xss')</script><p>World</p>").unwrap();
        assert!(!result.contains("script"));
        assert!(!result.contains("alert"));
        assert!(result.contains("Hello"));
        assert!(result.contains("World"));
    }

    #[test]
    fn removes_iframe() {
        let result = sanitize_html(r#"<p>Hi</p><iframe src="evil.com"></iframe>"#).unwrap();
        assert!(!result.contains("iframe"));
    }

    #[test]
    fn removes_event_handlers() {
        let result = sanitize_html(r#"<p onclick="alert('xss')">Hello</p>"#).unwrap();
        assert!(!result.contains("onclick"));
        assert!(!result.contains("alert"));
    }

    #[test]
    fn removes_javascript_urls() {
        let result = sanitize_html(r#"<a href="javascript:alert('xss')">Click</a>"#).unwrap();
        assert!(!result.contains("javascript:"));
    }

    #[test]
    fn preserves_safe_html() {
        let html = "<h1>Title</h1><p>Hello <strong>World</strong></p>";
        let result = sanitize_html(html).unwrap();
        assert_eq!(result, html);
    }

    #[test]
    fn rejects_oversized_input() {
        let huge = "x".repeat(MAX_INPUT_SIZE + 1);
        assert!(sanitize_html(&huge).is_err());
    }

    #[test]
    fn nesting_depth_check() {
        assert_eq!(check_nesting_depth("<a><b><c></c></b></a>"), 3);
        assert_eq!(check_nesting_depth("<p>Hello</p>"), 1);
    }

    #[test]
    fn rejects_excessive_nesting() {
        let html = "<div>".repeat(101) + &"</div>".repeat(101);
        let result = sanitize_html(&html);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("nesting depth"));
    }

    #[test]
    fn removes_self_closing_embed() {
        let result = sanitize_html(r#"<p>Hi</p><embed src="evil.swf" />"#).unwrap();
        assert!(!result.contains("embed"));
    }

    #[test]
    fn removes_unclosed_object_tag() {
        let result = sanitize_html(r#"<p>Hi</p><object data="evil.swf"><p>inner</p>"#).unwrap();
        assert!(!result.contains("object"));
    }

    #[test]
    fn removes_unquoted_event_handler() {
        let result = sanitize_html(r#"<p onclick=alert(1)>Hello</p>"#).unwrap();
        assert!(!result.contains("onclick"));
        assert!(result.contains("Hello"));
    }

    #[test]
    fn removes_form_tag() {
        let result = sanitize_html(r#"<form action="/submit"><input></form>"#).unwrap();
        assert!(!result.contains("form"));
    }

    #[test]
    fn removes_style_tag() {
        let result = sanitize_html(r#"<style>body { color: red }</style><p>Hi</p>"#).unwrap();
        assert!(!result.contains("style"));
        assert!(result.contains("Hi"));
    }

    #[test]
    fn unclosed_tag_no_gt() {
        // Tag with no closing > — hits the break in the else branch
        let result = sanitize_html("<p>Hi</p><embed src=x").unwrap();
        // Should handle gracefully
        assert!(result.contains("Hi"));
    }

    #[test]
    fn event_handler_with_whitespace_before_value() {
        let result = sanitize_html(r#"<div onmouseover = "alert(1)">Hi</div>"#).unwrap();
        assert!(!result.contains("onmouseover"));
        assert!(result.contains("Hi"));
    }
}
