use glob::Pattern;
use std::str;
pub(crate) enum GlobPatternType {
    Exact,
    EscapedExact,
    Star,
    NonStar,
}

pub(crate) fn analyze_glob_pattern(pattern: &[u8]) -> GlobPatternType {
    // Fast path for when none of the characters are present.
    if memchr::memchr3(b'*', b'?', b'\\', pattern).is_none()
        && memchr::memchr2(b'[', b']', pattern).is_none()
    {
        return GlobPatternType::Exact;
    }

    let mut pattern_type = GlobPatternType::Exact;
    let mut it = pattern.iter();

    while let Some(&c) = it.next() {
        match c {
            b'\\' => {
                // Found an escape sequence, mark as EscapedExact if no globs found yet
                if matches!(pattern_type, GlobPatternType::Exact) {
                    pattern_type = GlobPatternType::EscapedExact;
                }
                it.next();
            }
            b'*' => {
                return GlobPatternType::Star;
            }
            b'[' | b']' | b'?' => {
                pattern_type = GlobPatternType::NonStar;
            }
            _ => {}
        }
    }

    pattern_type
}

/// Unescapes a pattern by removing backslashes that escape special characters.
/// For exact patterns, we need to normalize escaped characters to their literal form.
pub(crate) fn unescape_pattern(pattern: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(pattern.len());
    let mut it = pattern.iter();

    while let Some(&c) = it.next() {
        if c == b'\\' {
            if let Some(&next_c) = it.next() {
                result.push(next_c);
            } else {
                // If backslash is at the end, include it as-is
                result.push(c);
            }
        } else {
            result.push(c);
        }
    }

    result
}

pub(crate) fn compile_glob_pattern(token: &[u8]) -> Result<Pattern, &'static str> {
    let pattern = str::from_utf8(token).map_err(|_| "Invalid UTF-8 string")?;
    // Right now, there is a pending PR that will support the '^' as the negation
    // character in the glob crate: https://github.com/rust-lang/glob/issues/116
    //
    // Let's optimistically assume the '^' cannot be part of the symbol's name (escaped
    // in a pattern).
    let pattern = pattern.replace("[^", "[!");
    Pattern::new(pattern.as_str()).map_err(|_| "Invalid Glob Pattern")
}

/// Normalizes repeated stars to a single wildcard outside character classes. Mach-O symbol
/// patterns do not use the glob crate's directory-recursion syntax, and backslashes are literal.
pub(crate) fn compile_macho_glob_pattern(token: &[u8]) -> Result<Pattern, &'static str> {
    let mut normalized = Vec::with_capacity(token.len());
    let mut bytes = token.iter().copied().peekable();

    while let Some(byte) = bytes.next() {
        normalized.push(byte);

        match byte {
            b'*' => while bytes.next_if_eq(&b'*').is_some() {},
            b'[' => {
                // Preserve character classes as-is.
                normalized.extend(bytes.next_if(|byte| matches!(byte, b'!' | b'^')));
                // A leading `]` (after any negation) is a literal member of the class.
                normalized.extend(bytes.next());
                for byte in bytes.by_ref() {
                    normalized.push(byte);
                    if byte == b']' {
                        break;
                    }
                }
            }
            _ => {}
        }
    }

    compile_glob_pattern(&normalized)
}

#[cfg(test)]
mod tests {
    use super::compile_macho_glob_pattern;

    #[test]
    fn macho_repeated_stars() {
        let pattern = compile_macho_glob_pattern(b"***_prefix***suffix**").unwrap();
        assert!(pattern.matches("_prefixsuffix"));
        assert!(pattern.matches("leading_prefix_middle_suffix_trailing"));
        assert!(!pattern.matches("_prefix_other"));
        assert!(!pattern.matches("_other_suffix"));
    }

    #[test]
    fn macho_stars_in_character_classes() {
        let pattern = compile_macho_glob_pattern(b"_class_[**]**").unwrap();
        assert!(pattern.matches("_class_*"));
        assert!(pattern.matches("_class_*_tail"));
        assert!(!pattern.matches("_class_x_tail"));
        assert!(!pattern.matches("_class_"));
    }

    #[test]
    fn macho_leading_close_bracket_in_character_classes() {
        let pattern = compile_macho_glob_pattern(b"_class_[]*]**").unwrap();
        assert!(pattern.matches("_class_]_tail"));
        assert!(pattern.matches("_class_*_tail"));
        assert!(!pattern.matches("_class_x_tail"));

        let negated = compile_macho_glob_pattern(b"_class_[!]*]**").unwrap();
        assert!(negated.matches("_class_x_tail"));
        assert!(!negated.matches("_class_]_tail"));
        assert!(!negated.matches("_class_*_tail"));
    }

    #[test]
    fn macho_backslashes_are_literal() {
        let pattern = compile_macho_glob_pattern(br"_prefix\**").unwrap();
        assert!(pattern.matches(r"_prefix\"));
        assert!(pattern.matches(r"_prefix\suffix"));
        assert!(!pattern.matches("_prefixsuffix"));
        assert!(!pattern.matches("_prefix*"));
    }
}
