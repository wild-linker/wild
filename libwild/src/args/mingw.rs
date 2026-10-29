#![allow(dead_code)]

use crate::arch::Architecture;
use crate::args::ArgumentParser;
use crate::args::Input;
use crate::args::InputSpec;
use crate::args::coff::CoffArgs;
use crate::args::msvc;
use crate::platform::Args;
use itertools::Itertools;
use std::path::Path;
use std::sync::Arc;
use strum::EnumMessage;
use strum::IntoEnumIterator;

// pub const SILENTLY_IGNORED_FLAGS: &[&str] = &[];

pub const IGNORED_FLAGS: &[&str] = &[];

// pub const IGNORED_NO_VALUE_FLAGS: &[&str] = &[];

pub(super) fn is_ignored_flag(flag: &str) -> bool {
    IGNORED_FLAGS.contains(&flag)
}

fn emulations() -> impl Iterator<Item = (Emulation, &'static str)> {
    Emulation::iter().flat_map(|emulation| {
        emulation
            .get_serializations()
            .iter()
            .map(move |&name| (emulation, name))
    })
}

#[derive(Debug, Clone, Copy, strum::EnumIter, strum::EnumMessage, strum::EnumString)]
pub(crate) enum Emulation {
    #[strum(serialize = "i386pep", message = "x86-64 PE target")]
    I386Pep,
    // TODO: add arm mess
    #[strum(disabled)]
    Unsupported,
}

impl Emulation {
    fn architecture(self) -> Architecture {
        match self {
            Emulation::I386Pep => Architecture::X86_64,
            // Emulation::I386Pe =>
            Emulation::Unsupported => Architecture::Unsupported,
        }
    }
}

pub(crate) fn supported_emulations() -> String {
    emulations().map(|(_, name)| name).join(" ")
}

pub(super) fn setup_argument_parser() -> ArgumentParser<CoffArgs> {
    let mut parser = ArgumentParser::<CoffArgs>::new();

    // TODO: copied from ELF
    parser
        .declare_with_param()
        .prefix("L")
        .help("Add directory to library search path")
        .execute(|args, _modifier_stack, value| {
            args.common_mut().save_dir.handle_file(value);
            args.lib_search_path.push(Box::from(Path::new(value)));
            Ok(())
        });

    parser
        .declare_with_param()
        .prefix("l")
        .help("Link with library")
        .sub_option_with_value(
            ":filename",
            "Link with specific file",
            |args, modifier_stack, value| {
                let stripped = value.strip_prefix(':').unwrap_or(value);
                let spec = InputSpec::File(Box::from(Path::new(stripped)));
                args.common_mut().inputs.push(Input {
                    spec,
                    search_first: None,
                    modifiers: *modifier_stack.last().unwrap(),
                });
                Ok(())
            },
        )
        .sub_option_with_value(
            "libname",
            // TODO: Improve the text.
            // TODO: Confirm the extension order.
            "Link with library libname.<extension>, with extension being: dll.a, a, lib, dll",
            |args, modifier_stack, value| {
                let spec = InputSpec::Lib(Box::from(value));
                args.common_mut().inputs.push(Input {
                    spec,
                    search_first: None,
                    modifiers: *modifier_stack.last().unwrap(),
                });
                Ok(())
            },
        )
        .execute(|args, modifier_stack, value| {
            let spec = if let Some(stripped) = value.strip_prefix(':') {
                InputSpec::Search(Box::from(stripped))
            } else {
                InputSpec::Lib(Box::from(value))
            };
            args.common_mut().inputs.push(Input {
                spec,
                search_first: None,
                modifiers: *modifier_stack.last().unwrap(),
            });
            Ok(())
        });

    // TODO: implement
    let mut emulation_option = parser
        .declare_with_param()
        .prefix("m")
        .help("Select linker emulation");

    for (emulation, name) in emulations() {
        emulation_option = emulation_option.value_help(name, emulation.get_message().unwrap());
    }

    emulation_option.execute(|args, _modifier_stack, value| {
        args.set_emulation_str(value, "-m");
        Ok(())
    });

    parser
        .declare_with_param()
        .prefix("O")
        .execute(|_args, _modifier_stack, _value|
            // We don't use opt-level for now.
            Ok(()));

    parser
        .declare()
        .long("static")
        .long("Bstatic")
        .help("Disallow linking of shared libraries")
        .execute(|_args, modifier_stack| {
            modifier_stack.last_mut().unwrap().allow_shared = false;
            Ok(())
        });

    parser
        .declare()
        .long("Bdynamic")
        .help("Allow linking of shared libraries")
        .execute(|_args, modifier_stack| {
            modifier_stack.last_mut().unwrap().allow_shared = true;
            Ok(())
        });

    parser
        .declare_with_param()
        .long("output")
        .prefix("o")
        .help("Set the output filename")
        .execute(|args, _modifier_stack, value| {
            args.common.output = Arc::from(Path::new(value));
            Ok(())
        });

    // TODO: port MSVC leftovers
    // parser
    //     .declare_with_param()
    //     .long("entry")
    //     .help("Set the entry point symbol")
    //     .execute(|args, _modifier_stack, value| {
    //         args.entry = Some(value.to_owned());
    //         Ok(())
    //     });
    //
    // parser
    //     .declare_with_param()
    //     .long("subsystem")
    //     .help("Set the subsystem, optionally with a version")
    //     .execute(|args, _modifier_stack, value| {
    //         args.subsystem = Some(parse_subsystem(value)?);
    //         Ok(())
    //     });
    //
    // parser
    //     .declare()
    //     .long("dll")
    //     .help("Build a DLL rather than an executable")
    //     .execute(|args, _modifier_stack| {
    //         args.is_dll = true;
    //         Ok(())
    //     });
    //
    // parser
    //     .declare_with_param()
    //     .long("machine")
    //     .help("Set the target machine")
    //     .execute(|args, _modifier_stack, value| {
    //         if !value.eq_ignore_ascii_case("x64") && !value.eq_ignore_ascii_case("amd64") {
    //             bail!("unsupported /MACHINE value `{value}`; only X64 is supported");
    //         }
    //         args.machine = CoffMachine::X86_64;
    //         Ok(())
    //     });
    //
    // parser
    //     .declare_with_param()
    //     .long("libpath")
    //     .help("Add directory to library search path")
    //     .execute(|args, _modifier_stack, value| {
    //         args.common.save_dir.handle_file(value);
    //         args.lib_search_path.push(Box::from(Path::new(value)));
    //         Ok(())
    //     });
    //
    // parser
    //     .declare_with_param()
    //     .long("defaultlib")
    //     .help("Add a default library")
    //     .execute(|args, _modifier_stack, value| {
    //         args.default_libraries.push(value.to_owned());
    //         Ok(())
    //     });
    //
    // // `/NODEFAULTLIB:lib` excludes one library, a bare `/NODEFAULTLIB` excludes all of them.
    // // Precedence over `/DEFAULTLIB` is applied when default libraries are resolved.
    // parser
    //     .declare_with_optional_param()
    //     .long("nodefaultlib")
    //     .help("Ignore one default library, or all of them if no library is named")
    //     .execute(|args, _modifier_stack, value| {
    //         match value {
    //             // An empty value is a malformed library name rather than a bare `/NODEFAULTLIB`.
    //             Some("") => bail!("missing value for /NODEFAULTLIB"),
    //             Some(value) => args.excluded_default_libraries.push(value.to_owned()),
    //             None => args.no_default_libraries = true,
    //         }
    //         Ok(())
    //     });
    //
    // // `/DEBUG:NONE` asks for no debug info, which is what we produce, so only the forms that
    // // request a PDB warn.
    // parser
    //     .declare_with_optional_param()
    //     .long("debug")
    //     .execute(|args, _modifier_stack, value| {
    //         match value {
    //             Some(value) if value.eq_ignore_ascii_case("none") => {}
    //             Some(value) => args.warn_unsupported(&format!("/DEBUG:{value}"))?,
    //             None => args.warn_unsupported("/DEBUG")?,
    //         }
    //         Ok(())
    //     });

    super::declare_common_args(&mut parser);

    add_silently_ignored_flags(&mut parser);

    parser
}

fn add_silently_ignored_flags(parser: &mut ArgumentParser<CoffArgs>) {
    for flag in msvc::SILENTLY_IGNORED_FLAGS {
        parser
            .declare_with_optional_param()
            .long(flag)
            .execute(|_args, _modifier_stack, _value| Ok(()));
    }

    for flag in msvc::IGNORED_NO_VALUE_FLAGS {
        parser
            .declare()
            .long(flag)
            .execute(|_args, _modifier_stack| Ok(()));
    }
}
