use crate::alignment::Alignment;
use crate::arch::Architecture;
use crate::args::CommonArgs;
use crate::args::Modifiers;
use crate::args::msvc;
use crate::error::Result;
use crate::platform;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CoffMachine {
    X86_64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Subsystem {
    Console,
    Windows,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct CoffArgs {
    pub(crate) is_mingw: bool,
    pub(crate) common: CommonArgs,
    pub(crate) entry: Option<String>,
    pub(crate) subsystem: Option<Subsystem>,
    pub(crate) is_dll: bool,
    pub(crate) lib_search_path: Vec<Box<Path>>,
    pub(crate) default_libraries: Vec<String>,
    /// Libraries named by `/NODEFAULTLIB:name`.
    pub(crate) excluded_default_libraries: Vec<String>,
    /// Set by a bare `/NODEFAULTLIB`, which ignores every default library.
    pub(crate) no_default_libraries: bool,

    pub(crate) machine: CoffMachine,
}

impl CoffArgs {
    pub(crate) fn new() -> Result<Self> {
        let args = Self {
            common: CommonArgs::from_env()?,
            ..Default::default()
        };

        Ok(args)
    }
}

impl Default for CoffArgs {
    fn default() -> Self {
        Self {
            is_mingw: false,
            common: CommonArgs::default(),
            entry: None,
            subsystem: None,
            is_dll: false,
            machine: CoffMachine::X86_64,
            lib_search_path: Vec::new(),
            default_libraries: Vec::new(),
            excluded_default_libraries: Vec::new(),
            no_default_libraries: false,
        }
    }
}

impl platform::Args for CoffArgs {
    fn parse<S, I>(&mut self, input: I) -> Result
    where
        S: AsRef<str>,
        I: Iterator<Item = S>,
    {
        parse(self, input)
    }

    fn should_strip_debug(&self) -> bool {
        todo!()
    }

    fn should_strip_all(&self) -> bool {
        false
    }

    fn entry_point<'a>(
        &'a self,
        _linker_script_entry: Option<&'a [u8]>,
    ) -> platform::EntryPoint<'a> {
        self.entry
            .as_deref()
            .map_or(platform::EntryPoint::None, |entry| {
                platform::EntryPoint::Symbol(entry.as_bytes())
            })
    }

    fn lib_search_path(&self) -> &[Box<Path>] {
        &self.lib_search_path
    }

    fn common(&self) -> &CommonArgs {
        &self.common
    }

    fn common_mut(&mut self) -> &mut CommonArgs {
        &mut self.common
    }

    fn should_export_all_dynamic_symbols(&self) -> bool {
        todo!()
    }

    fn should_export_dynamic(&self, _lib_name: &[u8]) -> bool {
        todo!()
    }

    fn loadable_segment_alignment(&self) -> Alignment {
        match self.architecture() {
            Architecture::X86_64 => Alignment { exponent: 12 },
            _ => unimplemented!(),
        }
    }

    fn should_merge_sections(&self) -> bool {
        // TODO
        true
    }

    fn should_output_executable(&self) -> bool {
        // TODO
        true
    }

    // TODO: use trait with implementations to avoid repeating `if mingw` all over the place?
    fn is_ignored_flag(&self, flag: &str) -> bool {
        msvc::is_ignored_flag(flag)
    }

    fn architecture(&self) -> Architecture {
        match self.machine {
            CoffMachine::X86_64 => Architecture::X86_64,
        }
    }
}

// Parse the supplied input arguments, which should not include the program name.
pub(crate) fn parse<S: AsRef<str>, I: Iterator<Item = S>>(
    args: &mut CoffArgs,
    mut input: I,
) -> Result {
    let mut modifier_stack = vec![Modifiers::default()];

    let arg_parser = msvc::setup_argument_parser();

    while let Some(arg) = input.next() {
        let arg = arg.as_ref();

        arg_parser.handle_argument(args, &mut modifier_stack, arg, &mut input)?;
    }

    args.common.report_unrecognized()?;

    Ok(())
}
