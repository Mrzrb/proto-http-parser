//! Utility functions

/// Convert string to snake_case
pub fn to_snake_case(input: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = input.chars().collect();
    
    for (i, &ch) in chars.iter().enumerate() {
        if ch.is_uppercase() {
            let should_add_underscore = if i == 0 {
                false
            } else {
                let prev_char = chars[i - 1];
                let next_char = chars.get(i + 1);
                
                // Add underscore if:
                // 1. Previous char was lowercase
                // 2. Previous char was uppercase and next char is lowercase (for cases like XMLHttp -> XML_Http)
                prev_char.is_lowercase() || 
                (prev_char.is_uppercase() && next_char.is_some_and(|c| c.is_lowercase()))
            };
            
            if should_add_underscore {
                result.push('_');
            }
            result.push(ch.to_lowercase().next().unwrap());
        } else {
            result.push(ch);
        }
    }
    
    result
}

/// Convert string to camelCase
pub fn to_camel_case(input: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;
    
    for ch in input.chars() {
        if ch == '_' || ch == '-' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(ch.to_uppercase().next().unwrap());
            capitalize_next = false;
        } else {
            result.push(ch);
        }
    }
    
    result
}

/// Convert string to PascalCase
pub fn to_pascal_case(input: &str) -> String {
    let camel = to_camel_case(input);
    if let Some(first_char) = camel.chars().next() {
        first_char.to_uppercase().collect::<String>() + &camel[1..]
    } else {
        camel
    }
}

/// Validate Rust identifier
pub fn is_valid_rust_identifier(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    
    let mut chars = name.chars();
    let first = chars.next().unwrap();
    
    if !first.is_alphabetic() && first != '_' {
        return false;
    }
    
    chars.all(|c| c.is_alphanumeric() || c == '_')
}

/// Sanitize identifier for Rust code generation
pub fn sanitize_identifier(name: &str) -> String {
    // Check if it's a Rust keyword
    let rust_keywords = [
        "as", "break", "const", "continue", "crate", "else", "enum", "extern",
        "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod",
        "move", "mut", "pub", "ref", "return", "self", "Self", "static", "struct",
        "super", "trait", "true", "type", "unsafe", "use", "where", "while",
        "async", "await", "dyn", "abstract", "become", "box", "do", "final",
        "macro", "override", "priv", "typeof", "unsized", "virtual", "yield", "try"
    ];
    
    if rust_keywords.contains(&name) || !is_valid_rust_identifier(name) {
        format!("r#{}", name)
    } else {
        name.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("HelloWorld"), "hello_world");
        assert_eq!(to_snake_case("XMLHttpRequest"), "xml_http_request");
        assert_eq!(to_snake_case("getUserById"), "get_user_by_id");
    }
    
    #[test]
    fn test_to_camel_case() {
        assert_eq!(to_camel_case("hello_world"), "helloWorld");
        assert_eq!(to_camel_case("get-user-by-id"), "getUserById");
    }
    
    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("hello_world"), "HelloWorld");
        assert_eq!(to_pascal_case("get-user-by-id"), "GetUserById");
    }
    
    #[test]
    fn test_is_valid_rust_identifier() {
        assert!(is_valid_rust_identifier("hello"));
        assert!(is_valid_rust_identifier("_private"));
        assert!(is_valid_rust_identifier("user_id"));
        assert!(!is_valid_rust_identifier("123invalid"));
        assert!(!is_valid_rust_identifier("hello-world"));
    }
    
    #[test]
    fn test_sanitize_identifier() {
        assert_eq!(sanitize_identifier("hello"), "hello");
        assert_eq!(sanitize_identifier("type"), "r#type");
        assert_eq!(sanitize_identifier("123invalid"), "r#123invalid");
        assert_eq!(sanitize_identifier("valid_name"), "valid_name");
    }
}