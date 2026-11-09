use crate::alignment::Alignment;
use crate::arch::Architecture;
use crate::args::CommonArgs;
use crate::args::Modifiers;
use crate::args::elf::Emulation;
use crate::args::mingw;
use crate::args::msvc;
use crate::error;
use crate::error::Error;
use crate::error::Result;
use crate::fs::FileSystem;
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

#[derive(Debug)]
pub(crate) enum CoffVariant {
    Mingw((Emulation, Option<Error>)),
    Msvc(CoffMachine),
}

#[derive(Debug)]
pub struct CoffArgs {
    pub(crate) common: CommonArgs,
    pub(crate) entry: Option<String>,
    pub(crate) subsystem: Option<Subsystem>,
    pub(crate) is_dll: bool,
    pub(crate) lib_search_path: Vec<Box<Path>>,
    pub(crate) default_libraries: Vec<String>,
    pub(crate) excluded_default_libraries: Vec<String>,
    pub(crate) no_default_libraries: bool,
    pub(crate) variant: CoffVariant,
}

impl CoffArgs {
    pub(crate) fn new(emulation: Option<Emulation>) -> Result<Self> {
        let variant = match emulation {
            Some(e) => CoffVariant::Mingw((e, None)),
            None => CoffVariant::Msvc(CoffMachine::X86_64),
        };
        let args = Self {
            variant,
            common: CommonArgs::from_env()?,
            ..Default::default()
        };

        Ok(args)
    }

    fn is_mingw(&self) -> bool {
        matches!(self.variant, CoffVariant::Mingw(_))
    }

    pub(crate) fn set_emulation_str(&mut self, value: &str, source: &'static str) {
        if let Ok(emulation) = value.parse() { self.set_emulation(emulation) } else {
            let emulation_error = Some(error!(
                "Emulation '{value}' is not yet supported (from {source})"
            ));
            self.variant = CoffVariant::Mingw((Emulation::Unsupported, emulation_error));
        }
    }

    fn set_emulation(&mut self, emulation: Emulation) {
        self.variant = CoffVariant::Mingw((emulation, None));
    }
}

impl Default for CoffArgs {
    fn default() -> Self {
        Self {
            common: CommonArgs::default(),
            entry: None,
            subsystem: None,
            is_dll: false,
            lib_search_path: Vec::new(),
            default_libraries: Vec::new(),
            excluded_default_libraries: Vec::new(),
            no_default_libraries: false,
            variant: CoffVariant::Msvc(CoffMachine::X86_64),
        }
    }
}

impl platform::Args for CoffArgs {
    fn parse<S, I>(&mut self, input: I, file_system: &impl FileSystem) -> Result
    where
        S: AsRef<str>,
        I: Iterator<Item = S>,
    {
        parse(self, input, file_system)
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
        if self.is_mingw() {
            mingw::is_ignored_flag(flag)
        } else {
            msvc::is_ignored_flag(flag)
        }
    }

    fn architecture(&self) -> Architecture {
        match self.variant {
            CoffVariant::Mingw((emulation, _)) => emulation.architecture(),
            CoffVariant::Msvc(_) => Architecture::X86_64,
        }
    }
}

// Parse the supplied input arguments, which should not include the program name.
pub(crate) fn parse<S: AsRef<str>, I: Iterator<Item = S>>(
    args: &mut CoffArgs,
    mut input: I,
    file_system: &impl FileSystem,
) -> Result {
    let mut modifier_stack = vec![Modifiers::default()];

    let arg_parser = if args.is_mingw() {
        mingw::setup_argument_parser()
    } else {
        msvc::setup_argument_parser()
    };

    while let Some(arg) = input.next() {
        let arg = arg.as_ref();

        arg_parser.handle_argument(args, &mut modifier_stack, arg, &mut input, file_system)?;
    }

    args.common.report_unrecognized()?;

    if let CoffVariant::Mingw((_, e)) = &mut args.variant
        && let Some(e) = e.take()
    {
        return Err(e);
    }

    Ok(())
}
