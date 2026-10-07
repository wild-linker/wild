use crate::error::Context as _;
use crate::error::Result;
use crate::glob_match::compile_macho_glob_pattern;
use crate::hash::PreHashed;
use crate::input_data::ScriptData;
use crate::linker_script::skip_comments_and_whitespace;
use crate::symbol::UnversionedSymbolName;
use crate::version_script::MatchRules;
use crate::version_script::ParsedSymbolMatcher;
use crate::version_script::SymbolLookupNameWrapper;
use crate::version_script::SymbolMatcher;
use crate::version_script::parse_matcher;
use winnow::BStr;
use winnow::Parser;

#[derive(Debug, Default)]
pub(crate) struct ExportList<'data> {
    rules: MatchRules<'data>,
    required_symbols: Vec<&'data [u8]>,
}

impl<'data> ExportList<'data> {
    pub(crate) fn parse_gnu(data: ScriptData<'data>) -> Result<Self> {
        parse_gnu_export_list
            .parse(BStr::new(data.raw))
            .context("Failed to parse symbol export list")
    }

    pub(crate) fn parse_macho(data: ScriptData<'data>) -> Result<Self> {
        let mut out = Self::default();

        for (line_index, line) in data.raw.split(|&byte| byte == b'\n').enumerate() {
            let line = line.split(|&byte| byte == b'#').next().unwrap_or_default();
            let line = line.trim_ascii();
            if line.is_empty() {
                continue;
            }

            // Backslashes are literal here. Apple ld and LLD disagree on escape semantics.
            let matcher = if line == b"*" {
                SymbolMatcher::MatchesAll
            } else if memchr::memchr3(b'*', b'?', b'[', line).is_some() {
                let pattern = compile_macho_glob_pattern(line)
                    .map_err(|error| crate::error!("{error}"))
                    .with_context(|| {
                        format!("Failed to parse symbol export list line {}", line_index + 1)
                    })?;

                if line.contains(&b'*') {
                    SymbolMatcher::StarGlob(pattern)
                } else {
                    SymbolMatcher::NonstarGlob(pattern)
                }
            } else {
                out.required_symbols.push(line);
                SymbolMatcher::Exact(line)
            };

            out.rules.push(ParsedSymbolMatcher::Single(matcher));
        }

        Ok(out)
    }

    // Based on Version Script counterpart
    pub(crate) fn contains(&self, name: &PreHashed<UnversionedSymbolName>) -> bool {
        let mut lookup_name = SymbolLookupNameWrapper::from_name(name);

        if self.rules.general.matches_exact(&mut lookup_name, false)
            || self.rules.cxx.matches_exact(&mut lookup_name, true)
        {
            return true;
        }

        for &non_star in &[true, false] {
            if self
                .rules
                .general
                .matches_glob(&mut lookup_name, non_star, false)
                || self
                    .rules
                    .cxx
                    .matches_glob(&mut lookup_name, non_star, true)
            {
                return true;
            }
        }

        self.rules.general.matches_all() || self.rules.cxx.matches_all()
    }

    /// Adds a matcher using GNU version-script syntax. This does not add a required Mach-O export
    /// root, even when the matcher is an exact name.
    pub(crate) fn add_symbol(&mut self, symbol: &'data str, without_semicolon: bool) -> Result<()> {
        let matcher = parse_matcher(&mut BStr::new(symbol), without_semicolon)?;
        self.rules.push(matcher);
        Ok(())
    }

    /// Exact entries from a Mach-O export list. These act as roots for archive loading and must
    /// resolve to definitions. GNU export lists and wildcard entries do not add roots here.
    pub(crate) fn required_symbols(&self) -> impl Iterator<Item = &[u8]> {
        self.required_symbols.iter().copied()
    }
}

fn parse_gnu_export_list<'input>(input: &mut &'input BStr) -> winnow::Result<ExportList<'input>> {
    let mut out = ExportList::default();

    skip_comments_and_whitespace(input)?;

    '{'.parse_next(input)?;

    loop {
        skip_comments_and_whitespace(input)?;

        if input.starts_with(b"};") {
            "};".parse_next(input)?;
            skip_comments_and_whitespace(input)?;
            break;
        }

        let matcher = parse_matcher(input, false)?;
        out.rules.push(matcher);
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input_data::ScriptData;

    #[test]
    fn parse_inline() {
        let data = ScriptData {
            raw: b"{ f*; \"bar\"; extern \"C++\" { baz; qux; }; };",
        };
        let export_list = ExportList::parse_gnu(data).unwrap();
        assert!(export_list.contains(&UnversionedSymbolName::prehashed(b"foo")));
        assert!(export_list.contains(&UnversionedSymbolName::prehashed(b"bar")));
        assert!(export_list.contains(&UnversionedSymbolName::prehashed(b"baz")));
        assert!(export_list.contains(&UnversionedSymbolName::prehashed(b"qux")));
        assert!(!export_list.contains(&UnversionedSymbolName::prehashed(b"not_exported")));
    }

    #[test]
    fn parse_multiline_with_comments() {
        let data = ScriptData {
            raw: b"{
                    # Single line comment
                    foo;
                    \"bar\"; # With a quote

                    /*
                    * And a C-style comment
                    */
                    baz*;

                    extern \"C++\" {
                        qux; # C++ symbol
                    };
                };",
        };
        let export_list = ExportList::parse_gnu(data).unwrap();
        assert!(export_list.contains(&UnversionedSymbolName::prehashed(b"foo")));
        assert!(export_list.contains(&UnversionedSymbolName::prehashed(b"bar")));
        assert!(export_list.contains(&UnversionedSymbolName::prehashed(b"baz-test")));
        assert!(export_list.contains(&UnversionedSymbolName::prehashed(b"qux")));
        assert!(!export_list.contains(&UnversionedSymbolName::prehashed(b"not_exported")));
    }

    #[test]
    fn externs() {
        let data = ScriptData {
            raw: b"{
                    extern \"C\" {
                        foo;
                    };
                    extern \"C++\" {
                        bar;
                    };
                };",
        };
        let export_list = ExportList::parse_gnu(data).unwrap();
        assert!(export_list.contains(&UnversionedSymbolName::prehashed(b"foo")));
        assert!(export_list.contains(&UnversionedSymbolName::prehashed(b"bar")));
        assert!(!export_list.contains(&UnversionedSymbolName::prehashed(b"not_exported")));
    }

    #[test]
    fn macho_patterns_do_not_use_version_script_syntax() {
        let data = ScriptData {
            raw: b"\"_quoted\"\nextern \"C++\" { foo; };\n",
        };
        let export_list = ExportList::parse_macho(data).unwrap();

        assert!(export_list.contains(&UnversionedSymbolName::prehashed(b"\"_quoted\"")));
        assert!(!export_list.contains(&UnversionedSymbolName::prehashed(b"_quoted")));
        assert!(export_list.contains(&UnversionedSymbolName::prehashed(
            b"extern \"C++\" { foo; };"
        )));
    }

    #[test]
    fn macho_parse_error_includes_line_number() {
        let data = ScriptData {
            raw: b"_valid\n[invalid\n",
        };
        let error = ExportList::parse_macho(data).unwrap_err();

        assert!(error.to_string().contains("line 2"));
    }
}
