//! Support for version scripts. Version scripts are used for attaching versions to symbols when
//! producing a shared object and for controlling which symbols do and don't get exported. Version
//! scripts are technically part of the linker script syntax, via the VERSION command, but are
//! generally passed via the --version-script flag instead. They can also sometimes be quite large.
//! For this reason, we have a separate parser for them.

use crate::bail;
use crate::error::Context as _;
use crate::error::Result;
use crate::glob_match::GlobPatternType;
use crate::glob_match::GlobSyntax;
use crate::glob_match::Pattern;
use crate::glob_match::analyze_glob_pattern;
use crate::glob_match::classify_glob_pattern;
use crate::glob_match::compile_pattern;
use crate::glob_match::unescape_pattern;
use crate::hash::PassThroughHasher;
use crate::hash::PreHashed;
use crate::input_data::ScriptData;
use crate::linker_script::skip_comments_and_whitespace;
use crate::symbol::UnversionedSymbolName;
use crate::timing_phase;
use hashbrown::HashMap;
use hashbrown::HashSet;
use symbolic_demangle::Demangle;
use symbolic_demangle::DemangleOptions;
use winnow::BStr;
use winnow::Parser;
use winnow::error::ContextError;
use winnow::error::FromExternalError;
use winnow::token::take_until;
use winnow::token::take_while;

#[derive(Debug, Default)]
pub(crate) struct MatchRules<'data> {
    pub(crate) general: BasicMatchRules<'data>,
    pub(crate) cxx: BasicMatchRules<'data>,
}

#[derive(Debug, Default)]
pub(crate) struct VersionBody<'data> {
    globals: MatchRules<'data>,
    locals: MatchRules<'data>,
}

#[derive(Debug, Default)]
pub(crate) struct Version<'data> {
    pub(crate) name: &'data [u8],
    pub(crate) parent_index: Option<u16>,
    pub(crate) version_body: VersionBody<'data>,
}

/// A general version script. See https://sourceware.org/binutils/docs/ld/VERSION.html
#[derive(Debug, Default)]
pub(crate) struct RegularVersionScript<'data> {
    versions: Vec<Version<'data>>,
    version_name_mapping: HashMap<&'data [u8], usize>,
}

/// An optimized version script for Rustc.
/// It declares all symbols as local except for the explicitly listed global symbols.
/// Only contains general (non-C++) exact symbol matchers.
/// Doesn't use actual versioning.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct RustVersionScript<'data> {
    pub(crate) global: Vec<&'data [u8]>,
}

#[derive(Debug)]
/// Possibly specialized version script.
/// See `RegularVersionScript` for the general case.
pub(crate) enum VersionScript<'data> {
    Regular(RegularVersionScript<'data>),
    Rust(RustVersionScript<'data>),
}

impl Default for VersionScript<'_> {
    fn default() -> Self {
        VersionScript::Regular(RegularVersionScript::default())
    }
}

#[derive(Debug, Clone)]
pub(crate) enum SymbolMatcher<'data> {
    // Exact match.
    Exact(&'data [u8]),
    // Exact match with escape sequences already unescaped.
    EscapedExact(Vec<u8>),
    // A glob pattern with a '*' token.
    StarGlob(Pattern),
    // A glob pattern without any '*' token.
    NonstarGlob(Pattern),
    /// Glob pattern equal to '*'
    MatchesAll,
}

#[derive(Debug, Default)]
pub(crate) struct BasicMatchRules<'data> {
    exact: HashSet<PreHashed<UnversionedSymbolName<'data>>, PassThroughHasher>,
    escaped_exact: HashSet<Vec<u8>>,
    star_globs: Vec<Pattern>,
    nonstar_globs: Vec<Pattern>,
    matches_all: bool,
}

impl<'data> BasicMatchRules<'data> {
    fn merge(&mut self, other: Self) {
        self.exact.extend(other.exact);
        self.escaped_exact.extend(other.escaped_exact);
        self.star_globs.extend(other.star_globs);
        self.nonstar_globs.extend(other.nonstar_globs);
        self.matches_all |= other.matches_all;
    }

    fn push(&mut self, pattern: SymbolMatcher<'data>) {
        match pattern {
            SymbolMatcher::MatchesAll => self.matches_all = true,
            SymbolMatcher::StarGlob(glob) => self.star_globs.push(glob),
            SymbolMatcher::NonstarGlob(glob) => self.nonstar_globs.push(glob),
            SymbolMatcher::Exact(exact) => {
                self.exact.insert(UnversionedSymbolName::prehashed(exact));
            }
            SymbolMatcher::EscapedExact(unescaped) => {
                self.escaped_exact.insert(unescaped);
            }
        }
    }

    #[inline]
    pub(crate) fn matches_exact(
        &self,
        lookup: &mut SymbolLookupNameWrapper,
        mangled: bool,
    ) -> bool {
        // Check normal exact matches first
        if !self.exact.is_empty() {
            if mangled {
                let demangled_name = lookup.get_demangled_name();
                // The creation of UnversionedSymbolName should be relatively cheap as we construct
                // it at most twice.
                if self
                    .exact
                    .contains(&UnversionedSymbolName::prehashed(demangled_name.as_bytes()))
                {
                    return true;
                }
            } else if self.exact.contains(lookup.name) {
                return true;
            }
        }

        // Check escaped exact matches
        if !self.escaped_exact.is_empty() {
            let symbol_bytes = if mangled {
                let demangled_name = lookup.get_demangled_name();
                demangled_name.as_bytes()
            } else {
                lookup.name.bytes()
            };

            if self.escaped_exact.contains(symbol_bytes) {
                return true;
            }
        }

        false
    }

    #[inline]
    pub(crate) fn matches_glob(
        &self,
        lookup: &mut SymbolLookupNameWrapper,
        non_star: bool,
        mangled: bool,
    ) -> bool {
        let mut globs = if non_star {
            self.nonstar_globs.iter().peekable()
        } else {
            self.star_globs.iter().peekable()
        };
        // Early exit before we actually demangle the name.
        if globs.peek().is_none() {
            return false;
        }

        let name = if mangled {
            lookup.get_demangled_name().as_bytes()
        } else {
            lookup.name.bytes()
        };

        globs.any(|pattern| pattern.matches(name))
    }

    #[inline]
    pub(crate) fn matches_all(&self) -> bool {
        self.matches_all
    }
}

enum VersionRuleSection {
    Global,
    Local,
}

#[derive(Debug)]
pub(crate) enum ParsedSymbolMatcher<'data> {
    Single(SymbolMatcher<'data>),
    Multiple(Vec<SymbolMatcher<'data>>),
    CxxMatchers(Vec<SymbolMatcher<'data>>),
}

impl<'data> MatchRules<'data> {
    pub(crate) fn merge(&mut self, other: Self) {
        self.general.merge(other.general);
        self.cxx.merge(other.cxx);
    }

    pub(crate) fn push(&mut self, pattern: ParsedSymbolMatcher<'data>) {
        match pattern {
            ParsedSymbolMatcher::Single(single) => {
                self.general.push(single);
            }
            ParsedSymbolMatcher::Multiple(matchers) => {
                for matcher in matchers {
                    self.general.push(matcher);
                }
            }
            ParsedSymbolMatcher::CxxMatchers(matchers) => {
                for matcher in matchers {
                    self.cxx.push(matcher);
                }
            }
        }
    }
}

pub(crate) struct SymbolLookupNameWrapper<'data> {
    name: &'data PreHashed<UnversionedSymbolName<'data>>,
    name_string: Option<&'data str>,
    demangled_name: Option<String>,
}

impl<'data> SymbolLookupNameWrapper<'data> {
    pub(crate) fn from_name(name: &'data PreHashed<UnversionedSymbolName<'data>>) -> Self {
        Self {
            name,
            name_string: None,
            demangled_name: None,
        }
    }

    pub(crate) fn get_name_string(&mut self) -> &'data str {
        self.name_string.get_or_insert_with(|| {
            str::from_utf8(self.name.bytes()).unwrap_or_else(|_| {
                panic!(
                    "Valid utf-8 identifier expected: {}",
                    String::from_utf8_lossy(self.name.bytes())
                )
            })
        })
    }

    pub(crate) fn get_demangled_name(&mut self) -> &String {
        // Extract the name string before the closure to avoid double mutable borrow
        let name_string = self.get_name_string();
        self.demangled_name.get_or_insert_with(|| {
            symbolic_common::Name::new(
                name_string,
                symbolic_common::NameMangling::Mangled,
                symbolic_common::Language::Cpp,
            )
            .demangle(DemangleOptions::complete().return_type(false))
            // Consider the original name if the demangler returns None.
            .unwrap_or_else(|| name_string.to_string())
        })
    }
}

impl<'data> RegularVersionScript<'data> {
    fn find_match(
        &self,
        name: &PreHashed<UnversionedSymbolName>,
    ) -> Option<(usize, VersionRuleSection)> {
        // Perform symbol lookup the same was as described for the LLD (and partially Mold) linker:
        // https://maskray.me/blog/2020-11-26-all-about-symbol-versioning#version-script
        let mut lookup_name = SymbolLookupNameWrapper::from_name(name);

        // 1) The first version tag with an exact pattern wins.
        for (i, version) in self.versions.iter().enumerate() {
            let body = &version.version_body;

            if body.globals.general.matches_exact(&mut lookup_name, false) {
                return Some((i, VersionRuleSection::Global));
            } else if body.locals.general.matches_exact(&mut lookup_name, false) {
                return Some((i, VersionRuleSection::Local));
            // Intentionally try first non-mangled names as it's much cheaper test.
            } else if body.globals.cxx.matches_exact(&mut lookup_name, true) {
                return Some((i, VersionRuleSection::Global));
            } else if body.locals.cxx.matches_exact(&mut lookup_name, true) {
                return Some((i, VersionRuleSection::Local));
            }
        }

        // 2) Otherwise, the last version tag with a non-* wildcard pattern wins ('global' should be
        //    checked first). Otherwise, the last version tag with a * pattern wins.
        for &non_star in &[true, false] {
            for (i, version) in self.versions.iter().enumerate().rev() {
                let body = &version.version_body;
                if body
                    .globals
                    .general
                    .matches_glob(&mut lookup_name, non_star, false)
                    || body
                        .globals
                        .cxx
                        .matches_glob(&mut lookup_name, non_star, true)
                {
                    return Some((i, VersionRuleSection::Global));
                } else if body
                    .locals
                    .general
                    .matches_glob(&mut lookup_name, non_star, false)
                    || body
                        .locals
                        .cxx
                        .matches_glob(&mut lookup_name, non_star, true)
                {
                    return Some((i, VersionRuleSection::Local));
                }
            }
        }

        // 3) Otherwise, the last version tag with match all (*).
        for (i, version) in self.versions.iter().enumerate().rev() {
            let body = &version.version_body;
            if body.globals.general.matches_all || body.globals.cxx.matches_all {
                return Some((i, VersionRuleSection::Global));
            } else if body.locals.general.matches_all || body.locals.cxx.matches_all {
                return Some((i, VersionRuleSection::Local));
            }
        }

        None
    }
}

fn parse_version_script<'input>(input: &mut &'input BStr) -> winnow::Result<VersionScript<'input>> {
    // List of version names in the script, used to map parent version to version indexes
    let mut version_names: Vec<&[u8]> = Vec::new();

    skip_comments_and_whitespace(input)?;

    // Simple version script, only defines symbols visibility
    // May be rust-style version script.
    if input.starts_with(b"{") {
        let version_body = parse_version_section(input)?;

        ";".parse_next(input)?;

        skip_comments_and_whitespace(input)?;

        return Ok(version_body.into());
    }

    // Multiple versions, won't be rust-style.
    let mut version_script = RegularVersionScript::default();

    // Base version placeholder
    version_names.push(b"");
    version_script.versions.push(Version::default());

    while !input.is_empty() {
        let name = parse_token(input)?;

        skip_comments_and_whitespace(input)?;

        let version_body = parse_version_section(input)?.into();

        let parent_name = take_until(0.., b';').parse_next(input)?;

        let parent_index = if parent_name.is_empty() {
            None
        } else {
            // We don't expect lots of versions, so a linear scan seems reasonable.
            Some(
                version_names
                    .iter()
                    .position(|v| v == &parent_name)
                    .ok_or_else(|| {
                        ContextError::from_external_error(
                            input,
                            VersionScriptError::UnknownParentVersion,
                        )
                    })? as u16,
            )
        };

        ";".parse_next(input)?;

        skip_comments_and_whitespace(input)?;

        version_names.push(name);
        version_script.versions.push(Version {
            name,
            parent_index,
            version_body,
        });
        version_script
            .version_name_mapping
            .insert(name, version_script.versions.len() - 1);
    }

    Ok(VersionScript::Regular(version_script))
}
impl<'data> VersionScript<'data> {
    pub(crate) fn parse(data: ScriptData<'data>) -> Result<VersionScript<'data>> {
        timing_phase!("Parse version script");

        parse_version_script
            .parse(BStr::new(data.raw))
            .context("Failed to parse version script")
    }

    pub(crate) fn is_empty(&self) -> bool {
        matches!(self, Self::Regular(script) if script.versions.is_empty())
    }

    pub(crate) fn version_count(&self) -> u16 {
        match self {
            VersionScript::Regular(script) => script.version_count(),
            VersionScript::Rust(_) => 0,
        }
    }

    pub(crate) fn version_for_symbol(
        &self,
        name: &PreHashed<UnversionedSymbolName>,
        version_name: Option<&[u8]>,
    ) -> Result<Option<object::elf::VersionIndex>> {
        match self {
            VersionScript::Regular(script) => script.version_for_symbol(name, version_name),
            VersionScript::Rust(_) => Ok(None),
        }
    }
}

impl<'data> RegularVersionScript<'data> {
    #[cfg(test)]
    fn parse(data: ScriptData<'data>) -> Result<RegularVersionScript<'data>> {
        match VersionScript::parse(data)? {
            VersionScript::Regular(script) => Ok(script),
            VersionScript::Rust(_) => {
                bail!("Rust-style version script cannot be used as a regular version script")
            }
        }
    }

    pub(crate) fn is_local(&self, name: &PreHashed<UnversionedSymbolName>) -> bool {
        self.find_match(name)
            .is_some_and(|(_, rule)| matches!(rule, VersionRuleSection::Local))
    }

    /// Number of versions in the Version Script, including the base version.
    pub(crate) fn version_count(&self) -> u16 {
        if self.versions.len() == 1 {
            // Ignore it if we have just the base version.
            0
        } else {
            self.versions.len() as u16
        }
    }

    pub(crate) fn version_iter(&self) -> impl Iterator<Item = &Version<'data>> {
        self.versions.iter()
    }

    pub(crate) fn version_for_symbol(
        &self,
        name: &PreHashed<UnversionedSymbolName>,
        version_name: Option<&[u8]>,
    ) -> Result<Option<object::elf::VersionIndex>> {
        let name_bytes = name.bytes();
        if let Some(version_name) = version_name {
            // There is a quirk that I couldn't find docs for. When a symbol has an empty version
            // (e.g. "foo@"), the versioning is disabled and the symbol has "hidden global version"
            // (visible as `1h <whitespaces>` in `readelf -V`), even if that symbol appears in the
            // version script.
            if version_name.is_empty() {
                return Ok(Some(object::elf::VER_NDX_GLOBAL));
            } else if let Some(&number) = self.version_name_mapping.get(version_name) {
                return Ok(Some(object::elf::VER_NDX_GLOBAL + number as u16));
            }
            bail!(
                "Symbol {} has undefined version {}",
                String::from_utf8_lossy(name_bytes),
                String::from_utf8_lossy(version_name),
            );
        }

        Ok(self.find_match(name).and_then(|(number, _)| {
            if number == 0 {
                // Ignore the implicit version!
                None
            } else {
                Some(object::elf::VER_NDX_GLOBAL + number as u16)
            }
        }))
    }
}

#[derive(Debug, Default)]
/// A generic parsed version body before version script specialization optimizations.
struct RawVersionBody<'data> {
    pub globals: Vec<ParsedSymbolMatcher<'data>>,
    pub locals: Vec<ParsedSymbolMatcher<'data>>,
}

impl<'data> RawVersionBody<'data> {
    fn rust_like(&self) -> bool {
        // one of the local has to be match-all `*` wildcard
        if !self.locals.iter().any(|matcher| {
            matches!(
                matcher,
                ParsedSymbolMatcher::Single(SymbolMatcher::MatchesAll)
            )
        }) {
            return false;
        }

        // and only exact matchers in global
        self.globals.iter().all(|matcher| {
            matches!(
                matcher,
                ParsedSymbolMatcher::Single(SymbolMatcher::Exact(_))
            )
        })
    }
}

impl<'data> TryFrom<RawVersionBody<'data>> for RustVersionScript<'data> {
    type Error = RawVersionBody<'data>;

    fn try_from(body: RawVersionBody<'data>) -> Result<Self, Self::Error> {
        if !body.rust_like() {
            return Err(body);
        }
        let global = body
            .globals
            .into_iter()
            .map(|matcher| {
                if let ParsedSymbolMatcher::Single(SymbolMatcher::Exact(name)) = matcher {
                    name
                } else {
                    unreachable!()
                }
            })
            .collect();

        Ok(RustVersionScript { global })
    }
}

impl<'data> From<RawVersionBody<'data>> for VersionScript<'data> {
    fn from(body: RawVersionBody<'data>) -> Self {
        match RustVersionScript::try_from(body) {
            Ok(rust_script) => VersionScript::Rust(rust_script),
            Err(body) => {
                let version_body = body.into();
                VersionScript::Regular(RegularVersionScript {
                    versions: vec![Version {
                        version_body,
                        ..Default::default()
                    }],
                    version_name_mapping: HashMap::new(),
                })
            }
        }
    }
}

impl<'data> From<RawVersionBody<'data>> for VersionBody<'data> {
    fn from(body: RawVersionBody<'data>) -> Self {
        let mut out = VersionBody::default();
        for global in body.globals {
            out.globals.push(global);
        }
        for local in body.locals {
            out.locals.push(local);
        }
        out
    }
}

#[inline(always)]
fn skip_ws_fast(input: &mut &BStr) -> winnow::Result<()> {
    let bytes = &**input;
    let mut i = 0;
    while i < bytes.len() && matches!(bytes[i], b' ' | b'\t' | b'\r' | b'\n') {
        i += 1;
    }
    *input = BStr::new(&bytes[i..]);
    if bytes.get(i) == Some(&b'#') || bytes[i..].starts_with(b"/*") {
        skip_comments_and_whitespace(input)?;
    }
    Ok(())
}

fn parse_version_section<'data>(input: &mut &'data BStr) -> winnow::Result<RawVersionBody<'data>> {
    let mut section = None;

    let mut out = RawVersionBody::default();

    '{'.parse_next(input)?;

    loop {
        skip_ws_fast(input)?;

        if try_take(input, b"}") {
            skip_ws_fast(input)?;
            break;
        }

        if try_take(input, b"global:") {
            section = Some(VersionRuleSection::Global);
        } else if try_take(input, b"local:") {
            section = Some(VersionRuleSection::Local);
        } else {
            let matcher = parse_matcher(input, false, true)?;

            match section {
                Some(VersionRuleSection::Global) | None => {
                    out.globals.push(matcher);
                }
                Some(VersionRuleSection::Local) => {
                    out.locals.push(matcher);
                }
            }
        }
    }

    Ok(out)
}

const CLASS_SAFE: u8 = 0;
const CLASS_GLOB: u8 = 1;
const CLASS_COLON: u8 = 2;
const CLASS_INVALID: u8 = 4;

/// Identifying and classifying a single byte according to the set of characters LLD accepts in
/// unquoted version script symbol names (`:` is valid only when paired as `::`).
const fn classify_byte(byte: u8) -> u8 {
    match byte {
        b'0'..=b'9'
        | b'A'..=b'Z'
        | b'a'..=b'z'
        | b'_'
        | b'.'
        | b'$'
        | b'/'
        | b'~'
        | b'='
        | b'+'
        | b'-'
        | b'!'
        | b'^' => CLASS_SAFE,
        b'*' | b'?' | b'[' | b']' | b'\\' => CLASS_GLOB,
        b':' => CLASS_COLON,
        _ => CLASS_INVALID,
    }
}

const BYTE_CLASS: [u8; 256] = {
    let mut table = [CLASS_INVALID; 256];
    let mut i = 0;
    while i < 256 {
        table[i] = classify_byte(i as u8);
        i += 1;
    }
    table
};

fn colons_are_paired(name: &[u8]) -> bool {
    let mut bytes = name.iter();
    while let Some(&byte) = bytes.next() {
        if byte == b':' && bytes.next() != Some(&b':') {
            return false;
        }
    }
    true
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UnquotedClass {
    Exact,
    HasGlob,
    Invalid,
}

// LLD allowed list of validation symbols
fn classify_unquoted_scalar(slice: &[u8]) -> UnquotedClass {
    let mut flags = 0u8;
    for &byte in slice {
        flags |= BYTE_CLASS[usize::from(byte)];
    }
    if flags & CLASS_INVALID != 0 {
        return UnquotedClass::Invalid;
    }
    if flags & CLASS_COLON != 0 && !colons_are_paired(slice) {
        return UnquotedClass::Invalid;
    }
    if flags & CLASS_GLOB != 0 {
        UnquotedClass::HasGlob
    } else {
        UnquotedClass::Exact
    }
}

#[inline(always)]
fn is_fast_plain_byte(byte: u8) -> bool {
    (byte | 0x20).wrapping_sub(b'a') < 26
        || byte.wrapping_sub(b'0') < 10
        || byte == b'_'
        || byte == b'.'
        || byte == b'$'
}

#[inline(always)]
fn chunk_all_plain(chunk: &[u8; 32]) -> bool {
    chunk
        .iter()
        .fold(true, |all, &byte| all & is_fast_plain_byte(byte))
}

fn find_non_plain_offset(token: &[u8]) -> Option<usize> {
    let (chunks, rem) = token.as_chunks::<32>();
    for (i, chunk) in chunks.iter().enumerate() {
        if !chunk_all_plain(chunk) {
            return Some(i * 32);
        }
    }
    if !rem.is_empty() {
        let tail: &[u8; 32] = token[token.len() - 32..].try_into().unwrap();
        if !chunk_all_plain(tail) {
            return Some(chunks.len() * 32);
        }
    }
    None
}

fn build_glob_matcher<'data>(
    token: &'data [u8],
    glob_type: GlobPatternType,
    input: &'data BStr,
) -> winnow::Result<SymbolMatcher<'data>> {
    let invalid_glob = |reason| {
        ContextError::from_external_error(&input, VersionScriptError::InvalidGlobPattern(reason))
    };

    Ok(match glob_type {
        GlobPatternType::Exact => SymbolMatcher::Exact(token),
        GlobPatternType::EscapedExact => {
            SymbolMatcher::EscapedExact(unescape_pattern(token).map_err(invalid_glob)?)
        }
        GlobPatternType::Star => {
            SymbolMatcher::StarGlob(compile_pattern(token, GlobSyntax::Gnu).map_err(invalid_glob)?)
        }
        GlobPatternType::NonStar => SymbolMatcher::NonstarGlob(
            compile_pattern(token, GlobSyntax::Gnu).map_err(invalid_glob)?,
        ),
    })
}

pub(crate) fn parse_matcher<'data>(
    input: &mut &'data BStr,
    without_semicolon: bool,     // e.g. symbol to export passed via CLI arg
    validate_symbol_names: bool, // e.g. names from a version script
) -> winnow::Result<ParsedSymbolMatcher<'data>> {
    if try_take(input, b"extern ") {
        let mut matchers = Vec::new();
        let cxx = if try_take(input, b"\"C++\"") {
            true
        } else if try_take(input, b"\"C\"") {
            false
        } else {
            let unsupported_extern: String = "{".parse_to().parse_next(input)?;
            return Err(ContextError::from_external_error(
                input,
                VersionScriptError::UnsupportedExtern(unsupported_extern),
            ));
        };
        skip_ws_fast(input)?;
        '{'.parse_next(input)?;

        loop {
            skip_ws_fast(input)?;

            if try_take(input, b"};") {
                skip_ws_fast(input)?;
                break;
            }

            // Symbols at the end of `extern` blocks may omit semicolons
            let expect_semicolon = {
                let remaining = &**input;
                if let Some(close_pos) = remaining.windows(2).position(|w| w == b"};") {
                    remaining[..close_pos].contains(&b';')
                } else {
                    without_semicolon
                }
            };

            let matcher = parse_matcher(input, !expect_semicolon, validate_symbol_names)?;
            let ParsedSymbolMatcher::Single(matcher) = matcher else {
                let unexpected_extern = if matches!(matcher, ParsedSymbolMatcher::CxxMatchers(_)) {
                    "C++"
                } else {
                    "C"
                };
                return Err(ContextError::from_external_error(
                    input,
                    VersionScriptError::UnexpectedExtern(unexpected_extern.to_string()),
                ));
            };

            matchers.push(matcher);
        }

        if cxx {
            return Ok(ParsedSymbolMatcher::CxxMatchers(matchers));
        }
        return Ok(ParsedSymbolMatcher::Multiple(matchers));
    }

    let token_start = *input;

    let token = if without_semicolon {
        let raw = if input.contains(&b'}') {
            take_until(1.., b'}').parse_next(input)?
        } else {
            // TODO: Clippy bug
            #[allow(clippy::needless_borrow)]
            &input
        };
        skip_comments_and_whitespace(input)?;
        try_take(input, b";");
        raw
    } else {
        let bytes = &**input;
        let Some(semi) = memchr::memchr(b';', bytes) else {
            return Err(ContextError::new());
        };
        if semi == 0 {
            return Err(ContextError::new());
        }
        *input = BStr::new(&bytes[semi + 1..]);
        &bytes[..semi]
    };

    let token = token.trim_ascii_end();

    if let Some(unquoted) = token.strip_circumfix(b"\"", b"\"") {
        return Ok(ParsedSymbolMatcher::Single(SymbolMatcher::Exact(unquoted)));
    }
    if token == b"*" {
        return Ok(ParsedSymbolMatcher::Single(SymbolMatcher::MatchesAll));
    }

    if validate_symbol_names {
        let (class, glob_offset) = if token.len() >= 32 {
            match find_non_plain_offset(token) {
                None => (UnquotedClass::Exact, token.len()),
                Some(offset) => (classify_unquoted_scalar(&token[offset..]), offset),
            }
        } else {
            (classify_unquoted_scalar(token), 0)
        };

        return match class {
            UnquotedClass::Exact => Ok(ParsedSymbolMatcher::Single(SymbolMatcher::Exact(token))),
            UnquotedClass::HasGlob => {
                let glob_type = classify_glob_pattern(&token[glob_offset..]);
                Ok(ParsedSymbolMatcher::Single(build_glob_matcher(
                    token, glob_type, input,
                )?))
            }
            UnquotedClass::Invalid => {
                //Error handling
                *input = token_start;
                Err(ContextError::from_external_error(
                    input,
                    VersionScriptError::InvalidSymbolName(
                        String::from_utf8_lossy(token).into_owned(),
                    ),
                ))
            }
        };
    }

    let glob_type = analyze_glob_pattern(token);
    Ok(ParsedSymbolMatcher::Single(build_glob_matcher(
        token, glob_type, input,
    )?))
}

/// Consumes `exact` from `input` or returns false if that's not what is next.
fn try_take(input: &mut &BStr, mut exact: &[u8]) -> bool {
    let result: Result<_, ContextError> = exact.parse_next(input);
    result.is_ok()
}

fn parse_token<'input>(input: &mut &'input BStr) -> winnow::Result<&'input [u8]> {
    take_while(1.., |b| !b" (){}\n\t".contains(&b)).parse_next(input)
}

#[derive(Debug)]
enum VersionScriptError {
    UnknownParentVersion,
    InvalidGlobPattern(&'static str),
    UnexpectedExtern(String),
    UnsupportedExtern(String),
    InvalidSymbolName(String),
}

impl std::error::Error for VersionScriptError {}

impl std::fmt::Display for VersionScriptError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VersionScriptError::InvalidGlobPattern(reason) => {
                write!(f, "Invalid glob pattern: {reason}")
            }
            VersionScriptError::UnknownParentVersion => write!(f, "Unknown parent version"),
            VersionScriptError::UnexpectedExtern(s) => {
                write!(f, "Unexpected extern \"{s}\" in parsing")
            }
            VersionScriptError::UnsupportedExtern(s) => write!(f, "Unsupported extern \"{s}\""),
            VersionScriptError::InvalidSymbolName(s) => {
                write!(
                    f,
                    "Symbol name \"{s}\" contains characters that require quoting"
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;
    use itertools::assert_equal;

    fn is_matching_global<'data>(script: &RegularVersionScript<'data>, name: &str) -> bool {
        let Some(m) = script.find_match(&UnversionedSymbolName::prehashed(name.as_bytes())) else {
            return true;
        };
        matches!(m.1, VersionRuleSection::Global)
    }

    #[test]
    fn parse_rust_version_script() {
        let data = ScriptData {
            raw: br"
                    {
                    global:
                        foo;
                        bar;
                    local:
                        *;
                    };",
        };
        let script = VersionScript::parse(data).unwrap();
        let VersionScript::Rust(rust_script) = script else {
            panic!("Expected Rust-style version script");
        };
        assert_equal(
            rust_script
                .global
                .iter()
                .map(|sym| String::from_utf8_lossy(sym)),
            ["foo", "bar"],
        );
    }

    #[test]
    fn test_parse_simple_version_script() {
        let data = ScriptData {
            raw: br"
                    # Comment starting with a hash
                    {global:
                        /* Single-line comment */
                        foo; /* Trailing comment */
                        bar*;
                        best_*_fn*;
                        *_wrapper  ;
                    local:
                        /* Multi-line
                           comment */
                        *;
                    };",
        };
        let script = RegularVersionScript::parse(data).unwrap();
        let version_body = &script.versions[0].version_body;
        assert_equal(
            version_body
                .globals
                .general
                .exact
                .iter()
                .map(|s| std::str::from_utf8(s.bytes()).unwrap()),
            ["foo"],
        );
        assert_equal(
            version_body
                .globals
                .general
                .star_globs
                .iter()
                .map(Pattern::source),
            [b"bar*".as_slice(), b"best_*_fn*", b"*_wrapper"],
        );

        assert!(is_matching_global(&script, "main_wrapper"));
        assert!(is_matching_global(&script, "bar_bar_bar"));
        assert!(is_matching_global(&script, "best_foo_fn_barus"));
        assert!(!is_matching_global(&script, "best_fn"));
    }

    #[test]
    fn test_parse_version_script() {
        let data = ScriptData {
            raw: br"
                VERS_1.1 {
                    global:
                        foo1;
                    local:
                        old*;
                };

                VERS_1.2 {
                    foo2;
                } VERS_1.1;
            ",
        };
        let script = RegularVersionScript::parse(data).unwrap();
        assert_eq!(script.versions.len(), 3);

        let version = &script.versions[1];
        assert_eq!(version.name, b"VERS_1.1");
        assert_eq!(version.parent_index, None);
        assert_equal(
            version
                .version_body
                .globals
                .general
                .exact
                .iter()
                .map(|s| std::str::from_utf8(s.bytes()).unwrap()),
            ["foo1"],
        );
        assert_equal(
            version
                .version_body
                .locals
                .general
                .star_globs
                .iter()
                .map(Pattern::source),
            [b"old*".as_slice()],
        );

        let version = &script.versions[2];
        assert_eq!(version.name, b"VERS_1.2");
        assert_eq!(version.parent_index, Some(1));
        assert_equal(
            version
                .version_body
                .globals
                .general
                .exact
                .iter()
                .map(|s| std::str::from_utf8(s.bytes()).unwrap()),
            ["foo2"],
        );
    }

    #[test]
    fn single_line_version_script() {
        let data = ScriptData {
            raw: br"VERSION42 { global: *; };",
        };
        RegularVersionScript::parse(data).unwrap();
    }

    #[test]
    fn extern_cxx_version_script() {
        let data = ScriptData {
            raw: br#"
                "VERSION42 {
                    local:
                        foo;
                        bar;
                        extern "C++" {
                            ns::*;
                            "f(int**,double)";
                            "std::vector<Loc<1>, std::allocator<Loc<1> > >::_M_realloc_append<Loc<1> const&>(Loc<1> const&)::_Guard_elts::_Guard_elts(Loc<1>*, std::allocator<Loc<1> >&)";
                            "WebKit::WebProcessMain(int, char**)";
                        };
                };"#,
        };
        let script = RegularVersionScript::parse(data).unwrap();
        let version_body = &script.versions[1].version_body;

        assert_equal(
            version_body
                .locals
                .cxx
                .exact
                .iter()
                .map(|s| std::str::from_utf8(s.bytes()).unwrap())
                .sorted(),
            [
                "WebKit::WebProcessMain(int, char**)",
                "f(int**,double)",
                "std::vector<Loc<1>, std::allocator<Loc<1> > >::_M_realloc_append<Loc<1> const&>(Loc<1> const&)::_Guard_elts::_Guard_elts(Loc<1>*, std::allocator<Loc<1> >&)",
            ],
        );
        assert_equal(
            version_body
                .locals
                .cxx
                .star_globs
                .iter()
                .map(Pattern::source),
            [b"ns::*".as_slice()],
        );

        assert!(!is_matching_global(&script, "foo"));
        // Test "ns::" c++ namespace glob pattern.
        assert!(!is_matching_global(
            &script,
            "_ZN2ns8generateB5cxx11ENSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEb"
        ));
        // Test exact matches after C++ demangling.
        assert!(!is_matching_global(
            &script,
            "_ZZNSt6vectorI3LocILi1EESaIS1_EE17_M_realloc_appendIJRKS1_EEEvDpOT_EN11_Guard_eltsC2EPS1_RS2_"
        ));
        assert!(!is_matching_global(
            &script,
            "_ZN6WebKit14WebProcessMainEiPPc"
        ));
        assert!(is_matching_global(
            &script,
            "_ZTVN10__cxxabiv120__si_class_type_infoE"
        ));
    }

    #[test]
    fn extern_c_version_script() {
        let data = ScriptData {
            raw: br#"
                "VERSION42 {
                    local:
                        foo;
                        bar;
                        extern "C" {
                            baz;
                        };
                };"#,
        };
        let script = RegularVersionScript::parse(data).unwrap();
        let version_body = &script.versions[1].version_body;

        assert_equal(
            version_body
                .locals
                .general
                .exact
                .iter()
                .map(|s| std::str::from_utf8(s.bytes()).unwrap())
                .sorted(),
            ["bar", "baz", "foo"],
        );
    }

    #[test]
    fn extern_without_semicolon_version_script() {
        let data = ScriptData {
            raw: br#"
                {
                    extern "C" {
                        foo
                    };
                };"#,
        };
        let script = RegularVersionScript::parse(data).unwrap();
        let version_body = &script.versions[0].version_body;

        assert_equal(
            version_body
                .globals
                .general
                .exact
                .iter()
                .map(|s| std::str::from_utf8(s.bytes()).unwrap()),
            ["foo"],
        );

        let data = ScriptData {
            raw: br#"
                {
                    extern "C++" {
                        bar;
                        baz
                    };
                };"#,
        };
        let script = RegularVersionScript::parse(data).unwrap();
        let version_body = &script.versions[0].version_body;

        assert_equal(
            version_body
                .globals
                .cxx
                .exact
                .iter()
                .map(|s| std::str::from_utf8(s.bytes()).unwrap())
                .sorted(),
            ["bar", "baz"],
        );
    }

    #[test]
    fn invalid_version_scripts() {
        #[track_caller]
        fn assert_invalid(src: &str) {
            let data = ScriptData {
                raw: src.as_bytes(),
            };
            assert!(VersionScript::parse(data).is_err());
        }

        // Missing ';'
        assert_invalid("{}");
        assert_invalid("{*};");
        assert_invalid("{foo};");

        // Missing '}'
        assert_invalid("{foo;");
        assert_invalid("VER1 {foo;}; VER2 {bar;} VER1");

        // Missing parent version
        assert_invalid("VER2 {bar;} VER1;");
    }

    #[test]
    fn test_version_order() {
        let data = ScriptData {
            raw: br"
                VERS_1.1 {
                    foo;
                    foo?;
                    f*;
                    bar*;
                };

                VERS_1.2 {
                    foo*;
                    bar;
                } VERS_1.1;
            ",
        };
        let script = RegularVersionScript::parse(data).unwrap();
        let sym = UnversionedSymbolName::prehashed;

        // Exact match wins
        assert_eq!(script.find_match(&sym(b"foo")).unwrap().0, 1);
        assert_eq!(script.find_match(&sym(b"bar")).unwrap().0, 2);

        // Non-star match
        assert_eq!(script.find_match(&sym(b"foox")).unwrap().0, 1);

        // Star match
        assert_eq!(script.find_match(&sym(b"foo_bar")).unwrap().0, 2);
    }

    #[test]
    fn test_escape_sequences() {
        let data = ScriptData {
            raw: br"
                {
                    global:
                        foo\*bar;
                        baz\?;
                        foo1\\foo2;
                        foo3?foo4*;
                        b*;
                        f?;
                };
            ",
        };
        let script = RegularVersionScript::parse(data).unwrap();
        let version_body = &script.versions[0].version_body;

        let escaped_patterns: HashSet<&[u8]> = version_body
            .globals
            .general
            .escaped_exact
            .iter()
            .map(|v| v.as_slice())
            .collect();

        assert!(escaped_patterns.contains(&b"foo*bar"[..]));
        assert!(escaped_patterns.contains(&b"baz?"[..]));
        assert!(escaped_patterns.contains(&b"foo1\\foo2"[..]));

        let star_patterns: Vec<&[u8]> = version_body
            .globals
            .general
            .star_globs
            .iter()
            .map(Pattern::source)
            .collect();

        assert!(star_patterns.contains(&b"b*".as_slice()));
        assert!(star_patterns.contains(&b"foo3?foo4*".as_slice()));

        let nonstar_patterns: Vec<&[u8]> = version_body
            .globals
            .general
            .nonstar_globs
            .iter()
            .map(Pattern::source)
            .collect();

        assert!(nonstar_patterns.contains(&b"f?".as_slice()));
    }

    #[test]
    fn test_negation() {
        let data = ScriptData {
            raw: br"
                VERS_1.1 {
                    f*;
                };

                VERS_1.2 {
                    f[^o]*;
                } VERS_1.1;
            ",
        };
        let script = RegularVersionScript::parse(data).unwrap();
        let sym = UnversionedSymbolName::prehashed;

        assert_eq!(script.find_match(&sym(b"foo")).unwrap().0, 1);
        assert_eq!(script.find_match(&sym(b"fxxx")).unwrap().0, 2);
    }

    #[test]
    fn test_parse_unquoted_invalid_symbol_name() {
        for name in [
            "wasi:cli/run@0.2.4#run",
            "foo@VER",
            "foo bar",
            "foo(int)",
            "a:b",
            ":ab",
            "ab:",
            "a:::b",
        ] {
            let script = format!("{{ global: {name}; local: *; }};");
            let error = VersionScript::parse(ScriptData {
                raw: script.as_bytes(),
            })
            .unwrap_err();
            assert!(
                format!("{error:?}").contains("contains characters that require quoting"),
                "{name}: {error:?}"
            );
        }
    }

    #[test]
    fn test_parse_quoted_and_valid_symbol_names() {
        let script = br#"{ global: "wasi:cli/run@0.2.4#run"; "foo(int)"; ns::foo*; a::::b; _ZN3foo3barEv; local: *; };"#;
        let script = RegularVersionScript::parse(ScriptData { raw: script }).unwrap();
        assert!(is_matching_global(&script, "wasi:cli/run@0.2.4#run"));
        assert!(is_matching_global(&script, "_ZN3foo3barEv"));
    }

    #[test]
    fn test_parse_matcher_does_not_validate_symbol_names() {
        let mut input = BStr::new("wasi:cli/run@0.2.4#run");
        assert!(parse_matcher(&mut input, true, false).is_ok());
    }

    #[test]
    fn test_parse_unquoted_name_whitespace() {
        for name in ["foo ", "foo\t", "a::b ", "foo"] {
            let script = format!("{name};");
            let mut input = BStr::new(&script);
            assert!(
                parse_matcher(&mut input, false, true).is_ok(),
                "{name:?} should be accepted"
            );
        }

        for name in ["foo bar ", "a: ", "a :b "] {
            let script = format!("{name};");
            let mut input = BStr::new(&script);
            assert!(
                parse_matcher(&mut input, false, true).is_err(),
                "{name:?} should be rejected"
            );
        }
    }

    fn ref_is_valid_unquoted_symbol_name(name: &[u8]) -> bool {
        let mut after_colon = false;
        for &byte in name {
            if after_colon {
                if byte != b':' {
                    return false;
                }
                after_colon = false;
                continue;
            }
            if byte == b':' {
                after_colon = true;
                continue;
            }
            if !byte.is_ascii_alphanumeric()
                && !matches!(
                    byte,
                    b'_' | b'.'
                        | b'$'
                        | b'/'
                        | b'\\'
                        | b'~'
                        | b'='
                        | b'+'
                        | b'['
                        | b']'
                        | b'*'
                        | b'?'
                        | b'-'
                        | b'!'
                        | b'^'
                )
            {
                return false;
            }
        }
        !after_colon
    }

    #[test]
    fn test_byte_classification_tables_exhaustive() {
        let mut safe_count = 0usize;
        let mut glob_count = 0usize;
        let mut colon_count = 0usize;
        let mut invalid_count = 0usize;

        for byte in 0u8..=255 {
            let class = BYTE_CLASS[usize::from(byte)];
            assert_eq!(class, classify_byte(byte), "byte {byte:#x}");

            let expected_plain = byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'$');
            assert_eq!(
                is_fast_plain_byte(byte),
                expected_plain,
                "is_fast_plain_byte mismatch for byte {byte:#x}"
            );
            if expected_plain {
                assert_eq!(class, CLASS_SAFE);
            }

            let ref_valid_single = byte != b':' && ref_is_valid_unquoted_symbol_name(&[byte]);
            let is_glob = matches!(byte, b'*' | b'?' | b'[' | b']' | b'\\');
            match class {
                CLASS_SAFE => {
                    assert!(ref_valid_single && !is_glob, "byte {byte:#x}");
                    safe_count += 1;
                }
                CLASS_GLOB => {
                    assert!(ref_valid_single && is_glob, "byte {byte:#x}");
                    glob_count += 1;
                }
                CLASS_COLON => {
                    assert_eq!(byte, b':');
                    colon_count += 1;
                }
                CLASS_INVALID => {
                    assert!(!ref_valid_single && byte != b':');
                    invalid_count += 1;
                }
                other => panic!("unexpected class {other} for byte {byte:#x}"),
            }
        }

        assert_eq!(safe_count, 72);
        assert_eq!(glob_count, 5);
        assert_eq!(colon_count, 1);
        assert_eq!(invalid_count, 178);
    }

    #[test]
    fn test_unquoted_validation_against_reference() {
        fn check_token(token: &[u8]) {
            let expected_valid = ref_is_valid_unquoted_symbol_name(token);
            let expected_glob = analyze_glob_pattern(token);

            let scalar_class = classify_unquoted_scalar(token);
            assert_eq!(
                scalar_class != UnquotedClass::Invalid,
                expected_valid,
                "scalar validity mismatch for {:?}",
                String::from_utf8_lossy(token)
            );
            if expected_valid {
                let got_glob = match scalar_class {
                    UnquotedClass::Exact => GlobPatternType::Exact,
                    UnquotedClass::HasGlob => classify_glob_pattern(token),
                    UnquotedClass::Invalid => unreachable!(),
                };
                assert_eq!(
                    std::mem::discriminant(&got_glob),
                    std::mem::discriminant(&expected_glob),
                    "scalar glob mismatch for {:?}",
                    String::from_utf8_lossy(token)
                );
            }

            if token.len() >= 32 {
                let (fast_class, glob_offset) = match find_non_plain_offset(token) {
                    None => (UnquotedClass::Exact, token.len()),
                    Some(offset) => (classify_unquoted_scalar(&token[offset..]), offset),
                };
                assert_eq!(
                    fast_class,
                    scalar_class,
                    "fast path class mismatch for {:?}",
                    String::from_utf8_lossy(token)
                );
                if fast_class == UnquotedClass::HasGlob {
                    let got_glob = classify_glob_pattern(&token[glob_offset..]);
                    assert_eq!(
                        std::mem::discriminant(&got_glob),
                        std::mem::discriminant(&expected_glob),
                        "fast path glob mismatch for {:?}",
                        String::from_utf8_lossy(token)
                    );
                }
            }
        }

        for len in [
            0, 1, 2, 15, 16, 17, 31, 32, 33, 63, 64, 65, 96, 128, 4095, 4096, 4097,
        ] {
            let plain = vec![b'a'; len];
            check_token(&plain);

            let legacy: Vec<u8> = (0..len)
                .map(|i| match i % 5 {
                    0 => b'Z',
                    1 => b'9',
                    2 => b'_',
                    3 => b'.',
                    _ => b'$',
                })
                .collect();
            check_token(&legacy);

            for &pos in &[0, 15, 16, 17, 31, 32, 33, 63, 64, 65, len.saturating_sub(1)] {
                if pos >= len {
                    continue;
                }
                for &inject in &[b':', b'*', b'?', b'[', b'\\', b'@', b' ', b'"', 0xC3, b'/'] {
                    let mut buf = legacy.clone();
                    buf[pos] = inject;
                    check_token(&buf);
                }
                for run in [2usize, 3, 4] {
                    if pos + run <= len {
                        let mut buf = legacy.clone();
                        buf[pos..pos + run].fill(b':');
                        check_token(&buf);
                    }
                }
                if pos + 2 <= len {
                    let mut buf = legacy.clone();
                    buf[pos] = b'\\';
                    buf[pos + 1] = b'*';
                    check_token(&buf);
                }
                if pos + 3 <= len {
                    let mut buf = legacy.clone();
                    buf[pos] = b'\\';
                    buf[pos + 1] = b'\\';
                    buf[pos + 2] = b'*';
                    check_token(&buf);
                }
            }
        }

        for sample in [
            b"".as_slice(),
            b":",
            b"::",
            b":::",
            b"::::",
            b"a:",
            b":a",
            b"a::b",
            b"a:::b",
            b"ns000::cls000::method_00000",
            b"ns000::r8n0::f000*",
            b"/leading/slash",
            b"a/b~c=d+e-f!g^h",
            b"foo\\*bar",
            b"a\"b",
            b"caf\xC3\xA9",
        ] {
            check_token(sample);
        }
    }

    #[test]
    fn test_unquoted_symbol_edge_cases() {
        let mut input = BStr::new(";");
        assert!(parse_matcher(&mut input, false, true).is_err());

        let mut input = BStr::new(r#""" ;"#);
        assert!(matches!(
            parse_matcher(&mut input, false, true).unwrap(),
            ParsedSymbolMatcher::Single(SymbolMatcher::Exact(b""))
        ));

        let mut input = BStr::new(r#"" ;"#);
        assert!(parse_matcher(&mut input, false, true).is_err());

        let mut input = BStr::new(r#""foo"bar;"#);
        assert!(parse_matcher(&mut input, false, true).is_err());

        let script = VersionScript::parse(ScriptData {
            raw: b"{ global: /leading/slash; _ZN3foo$LT$bar$GT$17h1234567890abcdefE; local: *; };",
        })
        .unwrap();
        let VersionScript::Rust(rust_script) = script else {
            panic!("Expected Rust-style version script");
        };
        assert_eq!(
            rust_script.global,
            vec![
                b"/leading/slash".as_slice(),
                b"_ZN3foo$LT$bar$GT$17h1234567890abcdefE".as_slice(),
            ]
        );

        let invalid_extern = VersionScript::parse(ScriptData {
            raw: b"{ extern \"C\" { bad:name }; };",
        });
        assert!(invalid_extern.is_err());

        let valid_extern = VersionScript::parse(ScriptData {
            raw: b"{ extern \"C\" { good::name }; };",
        });
        assert!(valid_extern.is_ok());

        let long_escaped = format!("{}foo\\*bar", "a".repeat(40));
        let script_src = format!("{{ global: {long_escaped}; local: *; }};");
        let parsed = RegularVersionScript::parse(ScriptData {
            raw: script_src.as_bytes(),
        })
        .unwrap();
        let expected_unescaped = format!("{}foo*bar", "a".repeat(40));
        assert!(
            parsed.versions[0]
                .version_body
                .globals
                .general
                .escaped_exact
                .contains(expected_unescaped.as_bytes())
        );
    }
}
