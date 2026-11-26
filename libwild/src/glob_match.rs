use fixedbitset::FixedBitSet;
use regex_automata::Input;
use regex_automata::meta::Cache;
use regex_automata::meta::Regex;
use regex_syntax::hir::Class;
use regex_syntax::hir::ClassBytes;
use regex_syntax::hir::ClassBytesRange;
use regex_syntax::hir::Dot;
use regex_syntax::hir::Hir;
use regex_syntax::hir::HirKind;
use regex_syntax::hir::Look;
use regex_syntax::hir::Repetition;
use std::cell::RefCell;
use std::iter::Peekable;
use std::mem::take;
use std::ops::RangeInclusive;
use std::sync::Arc;
use thread_local::ThreadLocal;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GlobSyntax {
    Gnu,
    // Backslashes are always literal, as are closing brackets outside classes.
    MachO,
}

#[derive(Clone, Debug)]
pub(crate) struct Pattern {
    inner: Arc<PatternInner>,
}

#[derive(Debug)]
struct PatternInner {
    #[cfg(test)]
    source: Vec<u8>,
    regex: Regex,
    cache: ThreadLocal<RefCell<Cache>>,
}

impl Pattern {
    pub(crate) fn matches(&self, name: &[u8]) -> bool {
        // Keep one cache per pattern per thread: a cache created from an empty regex
        // cannot reliably be reused across patterns (https://github.com/rust-lang/regex/issues/1155).
        let cache = self
            .inner
            .cache
            .get_or(|| RefCell::new(self.inner.regex.create_cache()));
        self.inner
            .regex
            .is_match_with(&mut cache.borrow_mut(), &Input::new(name))
    }

    #[cfg(test)]
    pub(crate) fn source(&self) -> &[u8] {
        &self.inner.source
    }
}

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

pub(crate) fn unescape_pattern(pattern: &[u8]) -> Result<Vec<u8>, &'static str> {
    let mut unescaped = Vec::with_capacity(pattern.len());
    for item in parse_glob(pattern, GlobSyntax::Gnu)? {
        match item.kind() {
            HirKind::Literal(literal) => unescaped.extend_from_slice(&literal.0),
            HirKind::Look(Look::Start | Look::End) => {}
            _ => return Err("An escaped exact pattern cannot contain glob metacharacters"),
        }
    }
    Ok(unescaped)
}

/// Parses a byte glob into HIR fragments, anchored at both ends.
fn parse_glob(pattern: &[u8], syntax: GlobSyntax) -> Result<Vec<Hir>, &'static str> {
    let mut items = vec![Hir::look(Look::Start)];
    let mut literal = Vec::new();
    let mut bytes = pattern.iter().copied().peekable();

    while let Some(byte) = bytes.next() {
        let item = match byte {
            b'\\' if syntax == GlobSyntax::Gnu => {
                literal.push(bytes.next().ok_or("Trailing escape in glob pattern")?);
                continue;
            }
            b'?' => Hir::dot(Dot::AnyByte),
            b'*' => {
                while bytes.next_if_eq(&b'*').is_some() {}
                Hir::repetition(Repetition {
                    min: 0,
                    max: None,
                    greedy: true,
                    sub: Box::new(Hir::dot(Dot::AnyByte)),
                })
            }
            b'[' => {
                let members = parse_brackets(&mut bytes, syntax)?;
                let ranges = bitset_one_ranges(&members)
                    .map(|range| ClassBytesRange::new(*range.start() as u8, *range.end() as u8));
                Hir::class(Class::Bytes(ClassBytes::new(ranges)))
            }
            b']' if syntax == GlobSyntax::Gnu => {
                return Err("Unmatched closing bracket in glob pattern");
            }
            _ => {
                literal.push(byte);
                continue;
            }
        };

        // Flush a collected literal before we insert a new item!
        if !literal.is_empty() {
            items.push(Hir::literal(take(&mut literal)));
        }
        items.push(item);
    }
    if !literal.is_empty() {
        items.push(Hir::literal(literal));
    }
    items.push(Hir::look(Look::End));
    Ok(items)
}

/// Yields one inclusive range for each consecutive run of set bits.
fn bitset_one_ranges(members: &FixedBitSet) -> impl Iterator<Item = RangeInclusive<usize>> {
    let mut ones = members.ones().peekable();
    std::iter::from_fn(move || {
        let start = ones.next()?;
        let mut end = start;
        while ones.next_if_eq(&(end + 1)).is_some() {
            end += 1;
        }
        Some(start..=end)
    })
}

/// Parses a bracket class after the caller has consumed its opening `[`.
fn parse_brackets(
    input: &mut Peekable<impl Iterator<Item = u8>>,
    syntax: GlobSyntax,
) -> Result<FixedBitSet, &'static str> {
    const UNCLOSED_CLASS: &str = "Unclosed bracket class in glob pattern";
    const TRAILING_ESCAPE: &str = "Trailing escape in glob bracket class";

    let negate = input.next_if(|byte| matches!(byte, b'!' | b'^')).is_some();
    let mut members = FixedBitSet::with_capacity(256);
    let mut start = input.next().ok_or(UNCLOSED_CLASS)?;
    loop {
        if start == b'\\' && syntax == GlobSyntax::Gnu {
            start = input.next().ok_or(TRAILING_ESCAPE)?;
        }
        if input.next_if_eq(&b'-').is_some() {
            let end = match input.next().ok_or(UNCLOSED_CLASS)? {
                b']' => {
                    // A dash at the end of a class is literal.
                    members.insert(start as usize);
                    members.insert(b'-' as usize);
                    break;
                }
                b'\\' if syntax == GlobSyntax::Gnu => input.next().ok_or(TRAILING_ESCAPE)?,
                byte => byte,
            };
            if start <= end {
                members.insert_range(start as usize..end as usize + 1);
            }
        } else {
            members.insert(start as usize);
        }

        start = input.next().ok_or(UNCLOSED_CLASS)?;
        if start == b']' {
            break;
        }
    }

    if negate {
        members.toggle_range(..);
    }
    Ok(members)
}

pub(crate) fn compile_pattern(token: &[u8], syntax: GlobSyntax) -> Result<Pattern, &'static str> {
    let items = parse_glob(token, syntax)?;
    let regex = Regex::builder()
        .configure(Regex::config().utf8_empty(false))
        .build_from_hir(&Hir::concat(items))
        .map_err(|_| "Failed to compile glob pattern")?;
    Ok(Pattern {
        inner: Arc::new(PatternInner {
            #[cfg(test)]
            source: token.into(),
            regex,
            cache: ThreadLocal::new(),
        }),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn macho_repeated_stars() {
        let pattern = compile_pattern(b"***_prefix***suffix**", GlobSyntax::MachO).unwrap();
        assert!(pattern.matches(b"_prefixsuffix"));
        assert!(pattern.matches(b"leading_prefix_middle_suffix_trailing"));
        assert!(!pattern.matches(b"_prefix_other"));
        assert!(!pattern.matches(b"_other_suffix"));
    }

    #[test]
    fn macho_stars_in_character_classes() {
        let pattern = compile_pattern(b"_class_[**]**", GlobSyntax::MachO).unwrap();
        assert!(pattern.matches(b"_class_*"));
        assert!(pattern.matches(b"_class_*_tail"));
        assert!(!pattern.matches(b"_class_x_tail"));
        assert!(!pattern.matches(b"_class_"));
    }

    #[test]
    fn macho_leading_close_bracket_in_character_classes() {
        let pattern = compile_pattern(b"_class_[]*]**", GlobSyntax::MachO).unwrap();
        assert!(pattern.matches(b"_class_]_tail"));
        assert!(pattern.matches(b"_class_*_tail"));
        assert!(!pattern.matches(b"_class_x_tail"));

        let negated = compile_pattern(b"_class_[!]*]**", GlobSyntax::MachO).unwrap();
        assert!(negated.matches(b"_class_x_tail"));
        assert!(!negated.matches(b"_class_]_tail"));
        assert!(!negated.matches(b"_class_*_tail"));
    }

    #[test]
    fn macho_backslashes_are_literal() {
        let pattern = compile_pattern(br"_prefix\**", GlobSyntax::MachO).unwrap();
        assert!(pattern.matches(br"_prefix\"));
        assert!(pattern.matches(br"_prefix\suffix"));
        assert!(!pattern.matches(b"_prefixsuffix"));
        assert!(!pattern.matches(b"_prefix*"));
    }

    #[test]
    fn gnu_glob_errors() {
        assert!(parse_glob(br"prefix\", GlobSyntax::Gnu).is_err());
        assert!(parse_glob(b"prefix]", GlobSyntax::Gnu).is_err());

        assert!(parse_glob(b"[", GlobSyntax::Gnu).is_err());
        assert!(parse_glob(b"[!", GlobSyntax::Gnu).is_err());
        assert!(parse_glob(b"[a", GlobSyntax::Gnu).is_err());
        assert!(parse_glob(b"[a-", GlobSyntax::Gnu).is_err());
        assert!(parse_glob(b"[a-z", GlobSyntax::Gnu).is_err());
        assert!(parse_glob(br"[\", GlobSyntax::Gnu).is_err());
        assert!(parse_glob(br"[a-\", GlobSyntax::Gnu).is_err());
    }
}
