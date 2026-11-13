use crate::arch::SUPPORTED_TARGETS;
use crate::args::ArgumentParser;
use crate::args::Input;
use crate::args::InputSpec;
use crate::args::coff::CoffArgs;
use crate::args::elf::emulations;
use crate::args::elf::supported_emulations;
use crate::platform::Args;
use std::path::Path;
use std::sync::Arc;
use strum::EnumMessage;

pub const SILENTLY_IGNORED_FLAGS: &[&str] = &[];

pub const IGNORED_FLAGS: &[&str] = &[];

pub const IGNORED_NO_VALUE_FLAGS: &[&str] = &["gc-sections"];

pub(super) fn is_ignored_flag(flag: &str) -> bool {
    IGNORED_FLAGS.contains(&flag)
}

pub(super) fn setup_argument_parser() -> ArgumentParser<CoffArgs> {
    let mut parser = ArgumentParser::<CoffArgs>::new();

    // TODO: copied from ELF, figure out how to share
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
            "Link with library libname",
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

    parser
        .declare()
        .long("help")
        .help("Show this help message")
        .execute(|_args, _modifier_stack| {
            // The following listing is something autoconf detection relies on.
            setup_argument_parser().print_help_and_exit(&format!(
                "wild: supported targets: {SUPPORTED_TARGETS}\nwild: supported emulations: {}\n",
                supported_emulations()
            ))
        });

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

    parser
        .declare_with_param()
        .long("plugin-opt")
        .help("Pass options to the plugin")
        .execute(|_args, _modifier_stack, _value| {
            // TODO: Implement support for linker plugins.

            Ok(())
        });

    parser
        .declare_with_param()
        .long("plugin")
        .help("Load plugin")
        .execute(|args, _modifier_stack, value| {
            args.warn_unsupported(&format!("--plugin {value}"))?;
            Ok(())
        });

    parser
        .declare_with_param()
        .long("entry")
        .short("e")
        .help("Set the entry point")
        .execute(|args, _modifier_stack, value| {
            args.entry = Some(value.to_owned());
            Ok(())
        });

    super::declare_common_args(&mut parser);

    add_silently_ignored_flags(&mut parser);

    parser
}

fn add_silently_ignored_flags(parser: &mut ArgumentParser<CoffArgs>) {
    for flag in SILENTLY_IGNORED_FLAGS {
        parser
            .declare_with_optional_param()
            .long(flag)
            .execute(|_args, _modifier_stack, _value| Ok(()));
    }

    for flag in IGNORED_NO_VALUE_FLAGS {
        parser
            .declare()
            .long(flag)
            .execute(|_args, _modifier_stack| Ok(()));
    }
}
