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

#[derive(Clone, Copy)]
enum Unescape {
    /// Keep the escaped byte as itself. The result is compared literally.
    Literal,
    /// Quote `* ? [ ]` as a one-character class.
    QuoteGlobMeta,
}

pub(crate) fn unescape_pattern(pattern: &[u8]) -> Vec<u8> {
    rewrite_escapes(pattern, Unescape::Literal)
}

fn rewrite_escapes(pattern: &[u8], mode: Unescape) -> Vec<u8> {
    let quote = matches!(mode, Unescape::QuoteGlobMeta);
    let mut result = Vec::with_capacity(pattern.len());
    let mut i = 0;
    while i < pattern.len() {
        if pattern[i] != b'\\' {
            if quote
                && pattern[i] == b'['
                && let Some((negate, members, consumed)) = parse_bracket(&pattern[i..])
            {
                result.extend(render_bracket(negate, &members));
                i += consumed;
                continue;
            }
            result.push(pattern[i]);
            i += 1;
            continue;
        }

        let Some(escaped) = pattern.get(i + 1).copied() else {
            result.push(b'\\');
            break;
        };
        i += 2;
        if quote && matches!(escaped, b'*' | b'?' | b'[' | b']') {
            // `glob::Pattern` does not treat `\` as an escape. A one-character
            // class is the supported way to match these bytes literally.
            result.extend_from_slice(&[b'[', escaped, b']']);
        } else {
            result.push(escaped);
        }
    }
    result
}

fn parse_bracket(input: &[u8]) -> Option<(bool, [bool; 256], usize)> {
    if input.first() != Some(&b'[') {
        return None;
    }
    let mut i = 1;
    // The glob crate only treats `!` as negation, not `^`.
    // https://github.com/rust-lang/glob/issues/116
    let negate = matches!(input.get(i), Some(b'!' | b'^'));
    if negate {
        i += 1;
    }

    let mut members = [false; 256];
    let mut c = *input.get(i)?;
    i += 1;
    loop {
        if c == b'\\' {
            c = *input.get(i)?;
            i += 1;
        }
        let start = c;
        let range =
            input.get(i) == Some(&b'-') && matches!(input.get(i + 1), Some(n) if *n != b']');
        if range {
            i += 1;
            let mut end = *input.get(i)?;
            i += 1;
            if end == b'\\' {
                end = *input.get(i)?;
                i += 1;
            }
            if start <= end {
                for byte in start..=end {
                    members[byte as usize] = true;
                }
            }
        } else {
            members[start as usize] = true;
        }

        c = *input.get(i)?;
        i += 1;
        if c == b']' {
            return Some((negate, members, i));
        }
    }
}

fn render_bracket(negate: bool, members: &[bool; 256]) -> Vec<u8> {
    let any = members.iter().any(|member| *member);
    if !any {
        // An empty range matches nothing. Negated, it matches any one byte.
        return if negate {
            vec![b'?']
        } else {
            b"[z-a]".to_vec()
        };
    }
    if !negate && members[b'!' as usize] && members.iter().filter(|member| **member).count() == 1 {
        return vec![b'!'];
    }

    let mut body = Vec::new();
    if members[b']' as usize] {
        body.push(b']');
    }
    for byte in 0u8..=255 {
        if members[byte as usize] && !matches!(byte, b']' | b'!' | b'-') {
            body.push(byte);
        }
    }
    let has_bang = members[b'!' as usize];
    let has_dash = members[b'-' as usize];
    if !negate && body.is_empty() && has_bang && has_dash {
        body.extend_from_slice(b"-!");
    } else if has_bang {
        body.push(b'!');
        if has_dash {
            body.push(b'-');
        }
    } else if has_dash {
        body.push(b'-');
    }

    let mut out = Vec::with_capacity(body.len() + 3);
    out.push(b'[');
    if negate {
        out.push(b'!');
    }
    out.extend(body);
    out.push(b']');
    out
}

pub(crate) fn compile_glob_pattern(token: &[u8]) -> Result<Pattern, &'static str> {
    let pattern = rewrite_escapes(token, Unescape::QuoteGlobMeta);
    let pattern = str::from_utf8(&pattern).map_err(|_| "Invalid UTF-8 string")?;
    Pattern::new(pattern).map_err(|_| "Invalid Glob Pattern")
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

    let pattern = str::from_utf8(&normalized).map_err(|_| "Invalid UTF-8 string")?;
    let pattern = pattern.replace("[^", "[!");
    Pattern::new(pattern.as_str()).map_err(|_| "Invalid Glob Pattern")
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
