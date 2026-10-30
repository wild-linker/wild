//! Support for saving inputs for later use.

use crate::FileSystem;
use crate::archive::ArchiveEntry;
use crate::archive::ArchiveIterator;
use crate::bail;
use crate::env;
use crate::error::Context as _;
use crate::error::Result;
use crate::file_kind::FileKind;
use crate::input_data::FileLoader;
use crate::linker_script::LinkerScript;
use crate::make_executable;
use crate::platform;
use foldhash::HashSet;
use std::borrow::Cow;
use std::io::BufWriter;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug, Default)]
pub(crate) struct SaveDir(Option<SaveDirState>);

const SAVE_DIR_ENV: &str = "WILD_SAVE_DIR";
const SAVE_BASE_ENV: &str = "WILD_SAVE_BASE";
const SKIP_LINKING_ENV: &str = "WILD_SAVE_SKIP_LINKING";

const PRELUDE: &str = include_str!("save-dir-prelude.sh");

#[derive(Debug)]
struct SaveDirState {
    dir: PathBuf,
    args: Vec<String>,
    files_to_copy: HashSet<PathBuf>,
}

impl SaveDir {
    pub(crate) fn new<S: AsRef<str>, I: Iterator<Item = S>>(mut args: I) -> Result<Self> {
        let Some(dir) = save_dir_from_env()? else {
            return Ok(Self(None));
        };

        // Skip program name.
        args.next();

        Ok(Self(Some(SaveDirState::new(
            dir,
            args.map(|s| s.as_ref().to_owned()).collect(),
        ))))
    }

    pub(crate) fn finish(
        &self,
        input_data: &FileLoader<'_, impl FileSystem>,
        parsed_args: &impl platform::Args,
    ) -> Result {
        if let Some(state) = self.0.as_ref() {
            let mut files_to_copy = state.files_to_copy.clone();
            files_to_copy.extend(
                input_data
                    .loaded_files
                    .iter()
                    .map(|file| file.filename.clone()),
            );
            state.finish(files_to_copy.iter(), parsed_args, input_data)?;
        }
        Ok(())
    }

    pub(crate) fn handle_file(&mut self, arg: &str) {
        if let Some(state) = self.0.as_mut() {
            state.files_to_copy.insert(Path::new(arg).to_path_buf());
        }
    }
}

fn save_dir_from_env() -> Result<Option<PathBuf>> {
    if let Ok(d) = env::var(SAVE_DIR_ENV) {
        let dir = PathBuf::from(d);

        if dir.exists() {
            std::fs::remove_dir_all(&dir).with_context(|| {
                format!(
                    "Failed to delete `{}`. If you're running multiple link commands \
                         concurrently, try using {} instead.",
                    dir.display(),
                    SAVE_BASE_ENV
                )
            })?;
        }

        std::fs::create_dir_all(&dir).with_context(|| {
            format!(
                "Failed to create directory `{}` specified by {SAVE_DIR_ENV}",
                dir.display()
            )
        })?;

        return Ok(Some(dir));
    }

    if let Ok(d) = env::var(SAVE_BASE_ENV) {
        let base = PathBuf::from(d);
        std::fs::create_dir_all(&base).with_context(|| {
            format!(
                "Failed to create directory `{}` specified by {SAVE_BASE_ENV}",
                base.display()
            )
        })?;

        let mut counter = 0;

        loop {
            let subdir = base.join(counter.to_string());
            if std::fs::create_dir(&subdir).is_ok() {
                return Ok(Some(subdir));
            }
            counter += 1;
        }
    } else {
        Ok(None)
    }
}

impl SaveDirState {
    fn new(dir: PathBuf, args: Vec<String>) -> Self {
        SaveDirState {
            dir,
            args,
            files_to_copy: Default::default(),
        }
    }

    /// Finalise the save directory. Makes sure that all `filenames` have been copied, writes the
    /// `run-with` file and if the environment variable is set to indicate that we should skip
    /// linking, then exit.
    fn finish<'a, I: Iterator<Item = &'a PathBuf>>(
        &self,
        filenames: I,
        parsed_args: &impl platform::Args,
        input_data: &FileLoader<'_, impl FileSystem>,
    ) -> Result {
        for filename in filenames {
            self.copy_file(&std::path::absolute(filename)?, input_data)?;
        }

        for path in &input_data.resolved_framework_search_path {
            self.copy_file(&std::path::absolute(path)?, input_data)?;
        }

        let run_with_file = self.dir.join("run-with");
        self.write_args_file(&run_with_file, parsed_args, input_data)
            .with_context(|| format!("Failed to write `{}`", run_with_file.display()))?;

        if env::var(SKIP_LINKING_ENV).is_ok() {
            std::process::exit(0);
        }
        Ok(())
    }

    fn write_args_file(
        &self,
        run_file: &Path,
        args: &impl platform::Args,
        input_data: &FileLoader<'_, impl FileSystem>,
    ) -> Result {
        let framework_search_path = &input_data.resolved_framework_search_path;
        let mut file = std::fs::File::create(run_file)?;
        let mut out = BufWriter::new(&mut file);
        out.write_all(PRELUDE.as_bytes())?;

        write_env(&mut out, args)?;

        // Collect at-file setup code and exec args separately so we can emit setup
        // code before the exec line.
        let mut setup_buf: Vec<u8> = Vec::new();
        let mut args_buf: Vec<u8> = Vec::new();
        let mut at_file_counter = 0usize;
        let original_output_file = self.write_args(
            &self.args,
            input_data.resolved_sysroot.as_deref(),
            framework_search_path,
            &mut args_buf,
            &mut setup_buf,
            &mut at_file_counter,
            false,
        )?;

        for path in framework_search_path {
            let path = std::path::absolute(path)?;
            if self.output_path(&path).is_dir() {
                write_script_arg_separator(&mut args_buf)?;
                args_buf.write_all(b"-F")?;
                write_copied_file_arg(&mut args_buf, &path)?;
            }
        }

        out.write_all(&setup_buf)?;
        out.write_all(b"exec \"${L[@]}\"")?;
        out.write_all(&args_buf)?;
        out.write_all(b"\\\n  \"$@\"")?;

        if let Some(orig) = original_output_file {
            out.write_all(b"\n# Original output file: ")?;
            out.write_all(orig.as_bytes())?;
        }

        drop(out);
        make_executable(&file)?;
        Ok(())
    }

    /// Writes arguments to `out`.
    ///
    /// When `is_rsp_file` is false (writing the main run-with script), each response file
    /// (@-prefixed argument) is saved as a separate file in the save-dir with path substitutions
    /// applied. Otherwise, nested `@file` arguments are expanded inline so the response file
    /// content stays self-contained.
    fn write_args(
        &self,
        args: &[String],
        sysroot: Option<&Path>,
        framework_search_path: &[Box<Path>],
        out: &mut dyn Write,
        setup_out: &mut dyn Write,
        at_file_counter: &mut usize,
        is_rsp_file: bool,
    ) -> Result<Option<String>> {
        let mut args = args.iter();
        let mut original_output_file = None;

        // TODO: This code is shared between different linker flavors, but includes flavor-specific
        // flags. We should address this.

        while let Some(arg) = args.next() {
            // Response files were already read when the arguments were parsed, so an `@` argument
            // that isn't a file is an option value, e.g. `-rpath @loader_path/../lib`.
            if let Some(args_path) = arg.strip_prefix("@")
                && Path::new(args_path).is_file()
            {
                let args_from_file = crate::args::read_args_from_file(Path::new(args_path))?;

                if is_rsp_file {
                    // Expand nested response files inline into the current file.
                    self.write_args(
                        &args_from_file,
                        sysroot,
                        framework_search_path,
                        out,
                        setup_out,
                        at_file_counter,
                        true,
                    )?;
                } else {
                    // Save to a separate file and reference it via a temp variable.
                    let rsp_index = *at_file_counter;
                    *at_file_counter += 1;
                    let at_filename = format!("at-{rsp_index}.txt");
                    let at_path = self.dir.join(&at_filename);

                    {
                        let mut at_file = std::fs::File::create(&at_path)
                            .with_context(|| format!("Failed to create `{}`", at_path.display()))?;
                        let mut at_out = BufWriter::new(&mut at_file);
                        let mut noop_setup: Vec<u8> = Vec::new();
                        self.write_args(
                            &args_from_file,
                            sysroot,
                            framework_search_path,
                            &mut at_out,
                            &mut noop_setup,
                            at_file_counter,
                            true,
                        )?;
                        at_out.flush()?;
                    }

                    write!(
                        setup_out,
                        "RSP_{rsp_index}=$(mktemp)\n\
                         while IFS= read -r LINE || [ -n \"$LINE\" ]; do\n\
                           LINE=\"${{LINE//\\$D/$D}}\"\n\
                           LINE=\"${{LINE//\\$OUT/$OUT}}\"\n\
                           printf '%s\\n' \"$LINE\"\n\
                         done < \"$D/{at_filename}\" > \"$RSP_{rsp_index}\"\n\
                         trap \"rm -f \\\"$RSP_{rsp_index}\\\"\" EXIT\n"
                    )?;

                    write_script_arg_separator(out)?;
                    write!(out, "@$RSP_{rsp_index}")?;
                }
                continue;
            }

            // Explicit -F options are replaced by the resolved paths in the main script.
            if !framework_search_path.is_empty()
                && let Some(dir) = arg.strip_prefix("-F")
            {
                if dir.is_empty() {
                    args.next();
                }
                continue;
            }

            write_arg_separator(out, is_rsp_file)?;

            if let Some(mut path) = arg.strip_prefix("-o") {
                if path.is_empty() {
                    path = args.next().map_or_default(|s| s.as_str());
                }
                out.write_all(b"-o $OUT")?;
                original_output_file = Some(path.to_owned());
            } else if let Some(mut dir) = arg.strip_prefix("-L") {
                if dir.is_empty() {
                    dir = args.next().map_or_default(|s| s.as_str());
                }

                let dir = crate::linker_script::maybe_forced_sysroot(Path::new(dir), sysroot)
                    .unwrap_or_else(|| Box::from(Path::new(dir)));
                let dir = std::path::absolute(dir)?;
                out.write_all(b"-L")?;
                write_copied_file_arg(out, &dir)?;
            } else if let Some(forced_path) =
                crate::linker_script::maybe_forced_sysroot(Path::new(arg), sysroot)
            {
                let path = std::path::absolute(&forced_path)?;
                if self.output_path(&path).exists() {
                    write_copied_file_arg(out, &path)?;
                } else if is_rsp_file {
                    out.write_all(arg.as_bytes())?;
                } else {
                    for b in arg.bytes() {
                        if b" $\\".contains(&b) {
                            out.write_all(b"\\")?;
                        }
                        out.write_all(&[b])?;
                    }
                }
            } else {
                // If the arg contains '=', then check to see if what's after the '=' is a filename
                // that exists. If it does, use that.
                let maybe_path = if let Some(eq_index) = arg.find('=') {
                    let after_equals = &arg[eq_index + 1..];
                    if Path::new(after_equals).exists() {
                        out.write_all(&arg.as_bytes()[..=eq_index])?;
                        after_equals
                    } else {
                        arg.as_str()
                    }
                } else {
                    arg.as_str()
                };

                let path = std::path::absolute(maybe_path)?;
                if self.output_path(&path).exists() {
                    write_copied_file_arg(out, &path)?;
                } else if is_rsp_file {
                    // At-file content is consumed directly by the linker, not by a shell, so no
                    // shell escaping is needed.
                    out.write_all(maybe_path.as_bytes())?;
                } else {
                    for b in maybe_path.bytes() {
                        if b" $\\".contains(&b) {
                            out.write_all(b"\\")?;
                        }
                        out.write_all(&[b])?;
                    }
                }
            }
        }

        Ok(original_output_file)
    }

    fn output_path(&self, path: &Path) -> PathBuf {
        self.dir.join(to_output_relative_path(path))
    }

    /// Copies `source_path` to our output directory.
    fn copy_file(
        &self,
        source_path: &Path,
        input_data: &FileLoader<'_, impl FileSystem>,
    ) -> Result {
        let dest_path = self.output_path(source_path);

        if dest_path.exists() || !source_path.exists() {
            return Ok(());
        }

        // The parent directory might be an actual directory or it might be a symlink. Either way,
        // we want to copy it before we copy our file.
        if let Some(parent) = source_path.parent() {
            self.copy_file(parent, input_data)?;
        }

        // We need to check again if `dest_path` exists because paths containing ".." mean that
        // creating the parent of `dest_path` might actually have created `dest_path`.
        if dest_path.exists() {
            return Ok(());
        }

        let meta = std::fs::symlink_metadata(source_path)
            .with_context(|| format!("Failed to read metadata for `{}`", source_path.display()))?;

        if meta.is_dir() {
            std::fs::create_dir(&dest_path)
                .with_context(|| format!("Failed to create directory `{}`", dest_path.display()))?;
        } else if meta.is_symlink() {
            let directory = source_path.parent().context("Invalid path")?;
            let mut target = std::fs::read_link(source_path)
                .with_context(|| format!("Failed to read symlink `{}`", source_path.display()))?;

            if target.is_absolute() {
                self.copy_file(&target, input_data)?;
                target = make_relative_path(&target, directory).with_context(|| {
                    format!(
                        "Failed to make path `{}` relative to `{}` while copying symlink",
                        target.display(),
                        directory.display()
                    )
                })?;
            } else {
                let absolute_target = directory.join(&target);
                self.copy_file(&absolute_target, input_data)?;
            }

            if let Err(error) = create_symlink(&target, &dest_path) {
                // If we can't create a symlink, then fall back to copying. If that fails, then
                // return the error from when we tried to create the symlink.
                if std::fs::copy(&target, &dest_path).is_err() {
                    return Err(error);
                }
            }
        } else {
            if let Ok(data) = std::fs::read(source_path) {
                match FileKind::identify_bytes(&data, false) {
                    Ok(FileKind::ThinArchive) => {
                        self.handle_thin_archive(source_path, input_data)?;
                    }
                    Ok(FileKind::Text) => {
                        let is_in_sysroot = input_data.is_in_sysroot(source_path);

                        // We make paths in linker scripts relative, but only if they're not inside
                        // of the sysroot. If they're inside the sysroot, then they need to remain
                        // absolute and will be interpreted as relative to the sysroot when linking.
                        if !is_in_sysroot {
                            // We don't want to prevent the save-dir mechanism from working just
                            // because we failed to parse a linker script, so in case of failure, we
                            // fall through to just copying the file as-is.
                            if let Ok(updated_bytes) = make_linker_script_relative(
                                &data,
                                source_path,
                                input_data.resolved_sysroot.as_deref(),
                            ) {
                                std::fs::write(dest_path, updated_bytes)?;
                                return Ok(());
                            }
                        }
                    }
                    _ => {}
                }
            }

            // To save disk space, we first attempt to hard link the file. If that fails, then just
            // copy it.
            if std::fs::hard_link(source_path, &dest_path).is_err() {
                std::fs::copy(source_path, &dest_path).with_context(|| {
                    format!(
                        "Failed to copy `{}` to `{}`",
                        source_path.display(),
                        dest_path.display()
                    )
                })?;
            }
        }

        Ok(())
    }

    /// Copies the files listed by the thin archive.
    fn handle_thin_archive(
        &self,
        path: &Path,
        input_data: &FileLoader<'_, impl FileSystem>,
    ) -> Result {
        let file_bytes = std::fs::read(path)?;
        let parent_path = path.parent().unwrap();

        for entry in ArchiveIterator::from_archive_bytes(&file_bytes)? {
            match entry? {
                ArchiveEntry::Thin(entry) => {
                    let entry_path = entry.ident.as_path();
                    if entry_path.is_absolute() {
                        bail!(
                            "Thin archive `{}` contained absolute path `{}`",
                            path.display(),
                            entry_path.display()
                        );
                    }
                    let absolute_entry_path = parent_path.join(entry_path);

                    self.copy_file(&absolute_entry_path, input_data)?;
                }
                ArchiveEntry::Regular(_) => {}
            }
        }

        Ok(())
    }
}

fn create_symlink(target: &Path, dest_path: &Path) -> Result {
    crate::host::fs::create_symlink(target, dest_path).with_context(|| {
        format!(
            "Failed to symlink {} to {}",
            dest_path.display(),
            target.display()
        )
    })
}

fn make_linker_script_relative(
    bytes: &[u8],
    source_path: &Path,
    sysroot: Option<&Path>,
) -> Result<Vec<u8>> {
    let script = LinkerScript::parse(bytes, source_path)?;
    let script_dir = source_path.parent().context("Invalid path")?;
    let base_ptr = bytes.as_ptr() as usize;

    // Collect position-based edits by walking the AST directly. Each Arg node
    // borrows into the original `bytes` slice, so we can recover exact byte
    // offsets via pointer arithmetic. This avoids the overlap bug inherent in
    // global string replacement, where one replacement can modify text
    // introduced by an earlier replacement.
    let mut edits: Vec<(usize, usize, String)> = Vec::new();
    collect_path_edits(&script.commands, base_ptr, script_dir, sysroot, &mut edits)?;

    // Sort by offset descending so earlier byte positions remain valid as we
    // apply later edits.
    edits.sort_by_key(|a| std::cmp::Reverse(a.0));

    let mut text = String::from_utf8(bytes.to_owned())?;
    for (offset, len, replacement) in edits {
        text.replace_range(offset..offset + len, &replacement);
    }

    Ok(text.into_bytes())
}

/// Walk the parsed command tree and collect `(byte_offset, byte_len,
/// replacement_string)` edits for every path token that needs rewriting.
fn collect_path_edits(
    commands: &[crate::linker_script::Command],
    base_ptr: usize,
    script_dir: &Path,
    sysroot: Option<&Path>,
    edits: &mut Vec<(usize, usize, String)>,
) -> Result {
    for command in commands {
        match command {
            crate::linker_script::Command::Arg(arg) => {
                if arg.starts_with(b"-l") {
                    continue;
                }
                let path_str = std::str::from_utf8(arg).context("Path isn't valid UTF-8")?;
                let path = Path::new(path_str);

                let replacement = if path.is_absolute() {
                    let Some(rel) = make_relative_path(path, script_dir) else {
                        continue;
                    };
                    Some(rel.to_str().context("Path isn't valid UTF-8")?.to_owned())
                } else if sysroot.is_none() {
                    if let Some(forced) = crate::linker_script::maybe_forced_sysroot(path, None) {
                        if forced.is_absolute() {
                            let Some(rel) = make_relative_path(&forced, script_dir) else {
                                continue;
                            };
                            Some(rel.to_str().context("Path isn't valid UTF-8")?.to_owned())
                        } else {
                            Some(
                                forced
                                    .to_str()
                                    .context("Path isn't valid UTF-8")?
                                    .to_owned(),
                            )
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };

                if let Some(replacement) = replacement {
                    let offset = arg.as_ptr() as usize - base_ptr;
                    edits.push((offset, arg.len(), replacement));
                }
            }
            crate::linker_script::Command::Group(subs)
            | crate::linker_script::Command::AsNeeded(subs) => {
                collect_path_edits(subs, base_ptr, script_dir, sysroot, edits)?;
            }
            _ => {}
        }
    }
    Ok(())
}

/// Removes any backtracking components (`..`) and the next component from `path`
/// For example, `/a/b/../c` becomes `/a/c`.
/// The path must be absolute. Will return `None` if backtracking past the root is attempted.
fn normalize_abs_path(path: &Path) -> Option<PathBuf> {
    assert!(path.is_absolute());
    let mut out = PathBuf::new();
    for comp in path.components() {
        if comp.as_os_str() == ".." {
            if !out.pop() {
                return None;
            }
        } else {
            out.push(comp);
        }
    }
    Some(out)
}

/// Returns a relative path to reach `target` from `directory`. Both must be absolute paths.
/// Returns `None` if the paths are invalid (e.g. contain backtracking past the root).
fn make_relative_path(target: &Path, directory: &Path) -> Option<PathBuf> {
    assert!(target.is_absolute());
    assert!(directory.is_absolute());
    let mut out = PathBuf::new();

    let target = normalize_abs_path(target)?;
    let directory = normalize_abs_path(directory)?;

    let mut target_comps = target.components().peekable();
    let mut dir_comps = directory.components().peekable();

    // We consume identical components until they diverge.
    loop {
        match (target_comps.peek(), dir_comps.peek()) {
            // identical paths
            (None, None) => return Some(PathBuf::from(".")),
            (Some(t), Some(d)) if t == d => {
                target_comps.next();
                dir_comps.next();
            }
            _ => break,
        }
    }
    // Now we just have the components that differ.

    for _ in dir_comps {
        out.push("..");
    }

    out.extend(target_comps);

    Some(out)
}

/// Writes the separator used between arguments in the main run-with shell script.
fn write_script_arg_separator(out: &mut dyn Write) -> Result {
    out.write_all(b" \\\n  ")?;
    Ok(())
}

/// Writes the appropriate argument separator for the current output mode.
fn write_arg_separator(out: &mut dyn Write, is_at_file: bool) -> Result {
    if is_at_file {
        out.write_all(b"\n")?;
    } else {
        write_script_arg_separator(out)?;
    }
    Ok(())
}

fn write_copied_file_arg(out: &mut dyn Write, path: &Path) -> Result {
    out.write_all(b"$D/")?;
    out.write_all(to_output_relative_path(path).as_os_str().as_encoded_bytes())?;
    Ok(())
}

/// Returns where we should copy `path` to when we put it in our output directory.
fn to_output_relative_path(path: &Path) -> PathBuf {
    path.iter()
        .filter(|p| p.as_encoded_bytes() != b"/")
        .collect()
}

/// Saves certain environment variables into the script. We only propagate environment variables
/// that are known to affect the link or to be used for communication between the compiler and say
/// linker plugins.
fn write_env(out: &mut BufWriter<&mut std::fs::File>, args: &impl platform::Args) -> Result {
    for var in &[
        "COLLECT_GCC",
        "COLLECT_GCC_OPTIONS",
        crate::args::elf::LDEMULATION_ENV,
    ] {
        if let Ok(mut value) = env::var(var) {
            // COLLECT_GCC_OPTIONS has things like "-o /path/to/output-file" in it. Update these so
            // that we use the run-with scripts output file instead.
            if *var == "COLLECT_GCC_OPTIONS"
                && let Some(out) = args.output().to_str()
            {
                value = value.replace(out, "${OUT}");
            }
            out.write_all(b"export ")?;
            out.write_all(var.as_bytes())?;
            out.write_all(b"=\"")?;
            out.write_all(shell_escape_string(&value).as_bytes())?;
            out.write_all(b"\"\n")?;
        }
    }

    Ok(())
}

fn shell_escape_string(value: &'_ str) -> Cow<'_, str> {
    if !value.contains('\\') && !value.contains('\"') {
        return Cow::Borrowed(value);
    }
    Cow::Owned(value.replace('\\', "\\\\").replace('"', "\\\""))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "macho")]
    #[test]
    fn save_implicit_framework_search_path() -> Result {
        use crate::args::macho::MachOArgs;
        use crate::platform::Args as _;
        use std::process::Command;

        if !crate::host::os::IS_MACOS {
            return Ok(());
        }
        let temp = tempfile::tempdir()?;
        let source = temp.path().join("source");
        let search = source.join("Library/Frameworks");
        let framework_name = Path::new("Foo.framework/Foo");
        let framework = search.join(framework_name);
        std::fs::create_dir_all(framework.parent().unwrap())?;
        std::fs::write(&framework, b"framework contents")?;
        let saved = temp.path().join("saved");
        std::fs::create_dir(&saved)?;

        // Substitute a temporary default directory without recording -F or -syslibroot.
        // A real SDK would already be relocated through the saved -syslibroot argument.
        let parsed = MachOArgs::default();
        let framework_search_path = vec![search.into_boxed_path()];
        let state = SaveDirState::new(saved.clone(), vec!["-framework".into(), "Foo".into()]);
        let arena = colosseum::sync::Arena::new();
        let mut input_data = FileLoader::new(&arena, std::sync::Arc::new(crate::fs::OsFileSystem));
        input_data.resolved_framework_search_path = framework_search_path;
        state.finish([&framework].into_iter(), &parsed, &input_data)?;
        std::fs::remove_dir_all(source)?;

        let output = Command::new("bash")
            .arg(saved.join("run-with"))
            .args(["printf", "%s\\n"])
            .output()?;
        assert!(output.status.success());
        let replay_args = String::from_utf8(output.stdout)?;
        let mut replay = MachOArgs::default();
        replay.parse(replay_args.lines())?;
        let found = replay
            .framework_search_path
            .iter()
            .map(|path| path.join(framework_name))
            .find(|path| path.is_file())
            .context("Saved arguments do not locate the copied framework")?;
        assert!(found.starts_with(saved));
        assert_eq!(std::fs::read(found)?, b"framework contents");
        Ok(())
    }

    fn test_make_relative_path(target: &Path, directory: &Path) {
        let relative = make_relative_path(target, directory).unwrap();

        let result = normalize_abs_path(&directory.join(&relative)).unwrap();
        let factual_target = normalize_abs_path(target).unwrap();

        assert_eq!(
            result,
            factual_target,
            "from `{}` to `{}`: got `{}` (resolves to `{}`), expected to resolve to `{}`",
            directory.display(),
            target.display(),
            relative.display(),
            result.display(),
            factual_target.display()
        );
    }

    #[test]
    fn make_relative_path_works() {
        let cases = [
            ("/a/b/c", "/a/b"),
            ("/a/b/c/d", "/a/b"),
            ("/a/b/c", "/a/b/x"),
            ("/a/b/c/d", "/a/b/x/y"),
            ("/a/b/c/d/e", "/a/b/x/y"),
            ("/a/b/c", "/a/b/c"),
            ("/a/b/c/d", "/a/b/c"),
            ("/a/b/c", "/a/b/c/d"),
            ("/a/b/c/d", "/a/b/c/d"),
            ("/a/b", "/d/c/e"),
            (
                "/usr/lib/libm.so.6",
                "/usr/bin/../lib64/gcc/x86_64-pc-linux-gnu/15.2.1/../../../../lib64/libm.so",
            ),
        ];

        for (a, b) in cases {
            let a = PathBuf::from(a);
            let b = PathBuf::from(b);
            test_make_relative_path(&a, &b);
            test_make_relative_path(&b, &a);
        }
    }

    #[test]
    fn make_linker_script_relative_works() {
        let script = b"GROUP ( /usr/lib/libc.so =/lib/libvalue.a $SYSROOT/lib/libfoo.a )";
        let script_path = Path::new("/work/scripts/script.ld");

        // Without sysroot: forced sysroot paths are made relative
        let without_sysroot = make_linker_script_relative(script, script_path, None).unwrap();
        let without_str = std::str::from_utf8(&without_sysroot).unwrap();
        assert_eq!(
            without_str,
            "GROUP ( ../../usr/lib/libc.so ../../lib/libvalue.a ../../lib/libfoo.a )"
        );

        // With sysroot: forced sysroot paths remain untouched (evaluated against --sysroot=$D/...)
        let with_sysroot =
            make_linker_script_relative(script, script_path, Some(Path::new("/my/sysroot")))
                .unwrap();
        let with_str = std::str::from_utf8(&with_sysroot).unwrap();
        assert_eq!(
            with_str,
            "GROUP ( ../../usr/lib/libc.so =/lib/libvalue.a $SYSROOT/lib/libfoo.a )"
        );
    }

    /// Verify that overlapping path tokens don't corrupt each other.
    ///
    /// With global string replacement, `=/lib/libvalue.a` and `/lib/libvalue.a`
    /// share the substring `/lib/libvalue.a`. The second replacement could match
    /// inside text produced by the first, or the first could consume part of the
    /// second token. Position-based edits eliminate this class of bug.
    #[test]
    fn make_linker_script_relative_overlap() {
        // Forced path appears before its absolute counterpart.
        let script = b"GROUP ( =/lib/libvalue.a /lib/libvalue.a )";
        let script_path = Path::new("/work/scripts/script.ld");
        let result = make_linker_script_relative(script, script_path, None).unwrap();
        let result_str = std::str::from_utf8(&result).unwrap();
        assert_eq!(
            result_str,
            "GROUP ( ../../lib/libvalue.a ../../lib/libvalue.a )"
        );

        // Reversed order: absolute path before forced path.
        let script = b"GROUP ( /lib/libvalue.a =/lib/libvalue.a )";
        let result = make_linker_script_relative(script, script_path, None).unwrap();
        let result_str = std::str::from_utf8(&result).unwrap();
        assert_eq!(
            result_str,
            "GROUP ( ../../lib/libvalue.a ../../lib/libvalue.a )"
        );

        // $SYSROOT variant overlapping with a plain absolute path.
        let script = b"GROUP ( $SYSROOT/lib/libvalue.a /lib/libvalue.a )";
        let result = make_linker_script_relative(script, script_path, None).unwrap();
        let result_str = std::str::from_utf8(&result).unwrap();
        assert_eq!(
            result_str,
            "GROUP ( ../../lib/libvalue.a ../../lib/libvalue.a )"
        );
    }

    /// Verify that paths inside AS_NEEDED are also rewritten, and that -l
    /// entries are left untouched.
    #[test]
    fn make_linker_script_relative_as_needed() {
        let script = b"GROUP ( AS_NEEDED ( /usr/lib/libm.so -lc =/lib/librt.a ) )";
        let script_path = Path::new("/work/scripts/script.ld");
        let result = make_linker_script_relative(script, script_path, None).unwrap();
        let result_str = std::str::from_utf8(&result).unwrap();
        assert_eq!(
            result_str,
            "GROUP ( AS_NEEDED ( ../../usr/lib/libm.so -lc ../../lib/librt.a ) )"
        );
    }
}
