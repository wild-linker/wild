//! Filesystem abstraction used by the linker.
//!
//! The main output is exposed as a sized random-access byte buffer because linker writers fill
//! disjoint regions in parallel. Auxiliary outputs are written as complete byte slices.

use crate::error::Context as _;
use crate::error::Result;
use crate::host::fs::FilesystemKind;
use memmap2::MmapOptions;
use std::fs::File;
use std::io::ErrorKind;
use std::io::Write as _;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileReplacementMode {
    /// The existing output file, if any, will be unlinked (deleted) and a new file with the same
    /// name put in its place. Any hard links to the file will not be affected.
    UnlinkAndReplace,

    /// The existing output file, if any, will be edited in-place. Any hard links to the file will
    /// update accordingly. If the file is locked due to currently being executed, then our write
    /// will fail.
    UpdateInPlace,

    /// As for `UpdateInPlace`, but unlink and replace if the file is currently being executed
    /// or has hard links.
    UpdateInPlaceWithFallback,
}

#[derive(Debug, Clone, Copy)]
pub enum FileWriteMode {
    Mmap,
    BufferThenWrite,
}

#[derive(Debug, Clone, Copy)]
pub struct OutputOptions {
    pub size: u64,
    pub file_replacement_mode: FileReplacementMode,
    pub write_mode: Option<FileWriteMode>,
    pub fallocate: Option<bool>,
    pub madvise_huge_pages: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    File,
    Directory,
    Other,
}

/// An opened linker input. Implementations own the storage returned by [`InputFile::bytes`].
pub trait InputFileData: Send + Sync + std::fmt::Debug {
    fn bytes(&self) -> &[u8];

    /// Returns whether the input still has the same identity as when it was opened.
    fn verify_unchanged(&self) -> std::io::Result<bool> {
        Ok(true)
    }

    /// Hint that the file will not be read again.
    fn release_memory(&self) {}
}

/// A sized, random-access linker output.
pub trait OutputFileData: Send {
    /// Returns the complete output buffer for reading.
    fn bytes(&self) -> &[u8];

    /// Returns the output buffer for random-access writing.
    fn bytes_mut(&mut self) -> &mut [u8];

    /// Persist the bytes and apply final file attributes.
    fn finish(self) -> Result;

    /// Invalidate any OS caches that may have observed partially written output.
    fn invalidate(&mut self, _len: usize) {}
}

/// Filesystem services needed by the core linker.
///
/// # Examples
///
/// ```
/// use libwild::{FileSystem, FileType, InputFileData, Linker, OutputFileData, OutputOptions};
/// use object::write::{Object, StandardSection, Symbol, SymbolSection};
/// use object::{Architecture, BinaryFormat, Endianness, SymbolFlags, SymbolKind, SymbolScope};
/// use std::collections::HashMap;
/// use std::fs::File;
/// use std::mem;
/// use std::path::{Path, PathBuf};
/// use std::sync::{Arc, Mutex};
///
/// // A small in-memory filesystem backed by a dictionary of path -> contents.
/// #[derive(Clone, Default)]
/// pub(crate) struct InMemoryFileSystem {
///     pub(crate) files: Arc<Mutex<HashMap<PathBuf, Vec<u8>>>>,
/// }
///
/// #[derive(Debug)]
/// struct Input(Vec<u8>);
///
/// impl InputFileData for Input {
///     fn bytes(&self) -> &[u8] {
///         &self.0
///     }
/// }
///
/// struct Output {
///     path: PathBuf,
///     bytes: Vec<u8>,
///     files: Arc<Mutex<HashMap<PathBuf, Vec<u8>>>>,
/// }
///
/// impl OutputFileData for Output {
///     fn bytes(&self) -> &[u8] {
///         &self.bytes
///     }
///
///     fn bytes_mut(&mut self) -> &mut [u8] {
///         &mut self.bytes
///     }
///
///     fn finish(mut self) -> libwild::error::Result {
///         let data = mem::take(&mut self.bytes);
///         self.files
///             .lock()
///             .unwrap()
///             .insert(self.path.clone(), data);
///         Ok(())
///     }
/// }
///
/// impl FileSystem for InMemoryFileSystem {
///     type Input = Input;
///     type Output = Output;
///
///     fn open_input(
///         &self,
///         path: &Path,
///         _prepopulate_maps: bool,
///     ) -> libwild::error::Result<(Self::Input, Option<Arc<File>>)> {
///         let bytes = self
///             .files
///             .lock()
///             .unwrap()
///             .get(&path.to_path_buf())
///             .cloned()
///             .ok_or_else(|| libwild::error!("No such in-memory file: {}", path.display()))?;
///         Ok((Input(bytes), None))
///     }
///
///     fn file_type(&self, path: &Path) -> libwild::error::Result<FileType> {
///         if self
///             .files
///             .lock()
///             .unwrap()
///             .contains_key(&path.to_path_buf())
///         {
///             Ok(FileType::File)
///         } else {
///             Err(std::io::Error::new(
///                 std::io::ErrorKind::NotFound,
///                 "no such in-memory file",
///             )
///             .into())
///         }
///     }
///
///     fn canonicalize(&self, path: &Path) -> libwild::error::Result<PathBuf> {
///         Ok(path.to_path_buf())
///     }
///
///     fn rename_file(&self, path: &Path, new_path: &Path) -> libwild::error::Result<()> {
///         let mut guard = self.files.lock().unwrap();
///         let Some(data) = guard.remove(&path.to_path_buf()) else {
///             return Err(std::io::Error::new(
///                 std::io::ErrorKind::NotFound,
///                 "no such in-memory file",
///             )
///             .into());
///         };
///         guard.insert(new_path.to_path_buf(), data);
///
///         Ok(())
///     }
///
///     fn remove_file(&self, path: &Path) -> libwild::error::Result<()> {
///         Ok(self
///             .files
///             .lock()
///             .unwrap()
///             .remove(&path.to_path_buf())
///             .map(|_| ())
///             .ok_or_else(|| {
///                 std::io::Error::new(std::io::ErrorKind::NotFound, "no such in-memory file")
///             })?)
///     }
///
///     fn create_output(
///         &self,
///         path: Arc<Path>,
///         options: OutputOptions,
///     ) -> libwild::error::Result<Self::Output> {
///         let size = usize::try_from(options.size)
///             .map_err(|_| libwild::error!("output is too large for this platform"))?;
///         Ok(Output {
///             path: path.to_path_buf(),
///             bytes: vec![0; size],
///             files: Arc::clone(&self.files),
///         })
///     }
///
///     fn write_auxiliary(&self, path: &Path, bytes: &[u8]) -> libwild::error::Result {
///         self.files
///             .lock()
///             .unwrap()
///             .insert(path.to_path_buf(), bytes.to_vec());
///         Ok(())
///     }
/// }
///
/// fn create_main_object() -> object::write::Result<Vec<u8>> {
///     let mut object = Object::new(BinaryFormat::Elf, Architecture::X86_64, Endianness::Little);
///     let data = object.section_id(StandardSection::Data);
///     let symbol = object.add_symbol(Symbol {
///         name: b"foo".to_vec(),
///         value: 0,
///         size: 0,
///         kind: SymbolKind::Data,
///         scope: SymbolScope::Dynamic,
///         weak: false,
///         section: SymbolSection::Undefined,
///         flags: SymbolFlags::None,
///     });
///     object.add_symbol_data(symbol, data, &42_u32.to_le_bytes(), 4);
///     object.write()
/// }
///
/// fn run() -> libwild::error::Result {
///     let fs = InMemoryFileSystem::default();
///
///     fs.files
///         .lock()
///         .unwrap()
///         .insert(PathBuf::from("main.o"), create_main_object()?);
///
///     let arguments = [
///         "wild",
///         "-m",
///         "elf_x86_64",
///         "-shared",
///         "main.o",
///         "-o",
///         "libx.so",
///     ];
///     let get_arguments = || arguments.into_iter();
///     let mut args = libwild::Args::new(get_arguments)?;
///     args.parse(get_arguments)?;
///
///     let linker = Linker::with_file_system(fs.clone());
///     linker.run(&args)?;
///
///     let output = fs
///         .files
///         .lock()
///         .unwrap()
///         .get(Path::new("libx.so"))
///         .cloned()
///         .ok_or_else(|| libwild::error!("linker did not create libx.so"))?;
///     // std::fs::write("libx.so", &output)?;
///     Ok(())
/// }
/// ```
pub trait FileSystem: Send + Sync + 'static {
    type Input: InputFileData;
    type Output: OutputFileData;

    /// Opens an input and optionally requests that its pages be populated in advance.
    fn open_input(
        &self,
        path: &Path,
        prepopulate_maps: bool,
    ) -> Result<(Self::Input, Option<Arc<File>>)>;

    /// Returns the type of the file at `path`.
    fn file_type(&self, path: &Path) -> Result<FileType>;

    /// Resolves symbolic links and returns the canonical absolute path.
    fn canonicalize(&self, path: &Path) -> Result<PathBuf>;

    /// Removes a file.
    fn remove_file(&self, path: &Path) -> Result<()>;

    /// Rename an existing file to a new path.
    fn rename_file(&self, path: &Path, new_path: &Path) -> Result<()>;

    /// Creates the sized random-access output.
    fn create_output(&self, path: Arc<Path>, options: OutputOptions) -> Result<Self::Output>;

    /// Writes a complete auxiliary output.
    fn write_auxiliary(&self, path: &Path, bytes: &[u8]) -> Result;
}

/// The normal host operating-system filesystem.
#[derive(Debug, Default, Clone, Copy)]
pub struct OsFileSystem;

impl OsFileSystem {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

#[derive(Debug)]
pub struct OsInputFile {
    bytes: crate::host::fs::InputBytes,
    path: PathBuf,
    /// The modification timestamp of the input file just before we opened it. We expect our input
    /// files not to change while we're running.
    modification_time: std::time::SystemTime,
}

impl InputFileData for OsInputFile {
    fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    fn verify_unchanged(&self) -> std::io::Result<bool> {
        Ok(std::fs::metadata(&self.path)?.modified()? == self.modification_time)
    }

    fn release_memory(&self) {
        crate::host::fs::release_input_memory(&self.bytes);
    }
}

enum OsOutputBuffer {
    Mmap(memmap2::MmapMut),
    InMemory(Vec<u8>),
}

pub struct OsOutputFile {
    file: File,
    buffer: OsOutputBuffer,
    path: Arc<Path>,
}

impl OutputFileData for OsOutputFile {
    fn bytes(&self) -> &[u8] {
        match &self.buffer {
            OsOutputBuffer::Mmap(mmap) => mmap,
            OsOutputBuffer::InMemory(bytes) => bytes,
        }
    }

    fn bytes_mut(&mut self) -> &mut [u8] {
        match &mut self.buffer {
            OsOutputBuffer::Mmap(mmap) => mmap,
            OsOutputBuffer::InMemory(bytes) => bytes,
        }
    }

    fn finish(mut self) -> Result {
        if let OsOutputBuffer::InMemory(bytes) = &self.buffer {
            self.file
                .write_all(bytes)
                .with_context(|| format!("Failed to write to {}", self.path.display()))?;
        }

        // Making the file executable is best-effort only. For example if we're writing to a pipe or
        // something, it isn't going to work and that's OK.
        let _ = make_executable(&self.file);

        Ok(())
    }

    fn invalidate(&mut self, len: usize) {
        if let OsOutputBuffer::Mmap(output) = &mut self.buffer {
            crate::host::fs::invalidate_mapped_output(output, len);
        }
    }
}

impl FileSystem for OsFileSystem {
    type Input = OsInputFile;
    type Output = OsOutputFile;

    fn open_input(
        &self,
        path: &Path,
        prepopulate_maps: bool,
    ) -> Result<(Self::Input, Option<Arc<File>>)> {
        let file = File::open(path)
            .with_context(|| format!("Failed to open input file `{}`", path.display()))?;

        let modification_time = file
            .metadata()
            .and_then(|meta| meta.modified())
            .with_context(|| {
                format!("Failed to read file modification time `{}`", path.display())
            })?;

        let bytes = crate::host::fs::read_input(&file, path, prepopulate_maps)?;

        Ok((
            OsInputFile {
                bytes,
                path: path.to_owned(),
                modification_time,
            },
            Some(Arc::new(file)),
        ))
    }

    fn file_type(&self, path: &Path) -> Result<FileType> {
        let ty = std::fs::metadata(path)?.file_type();
        Ok(if ty.is_file() {
            FileType::File
        } else if ty.is_dir() {
            FileType::Directory
        } else {
            FileType::Other
        })
    }

    fn canonicalize(&self, path: &Path) -> Result<PathBuf> {
        Ok(std::fs::canonicalize(path)?)
    }

    fn remove_file(&self, path: &Path) -> Result<()> {
        Ok(std::fs::remove_file(path)?)
    }

    fn rename_file(&self, path: &Path, new_path: &Path) -> Result<()> {
        Ok(std::fs::rename(path, new_path)?)
    }

    fn create_output(&self, path: Arc<Path>, options: OutputOptions) -> Result<Self::Output> {
        let mut open_options = std::fs::OpenOptions::new();

        match options.file_replacement_mode {
            FileReplacementMode::UnlinkAndReplace => {
                open_options.truncate(true);
            }
            FileReplacementMode::UpdateInPlace | FileReplacementMode::UpdateInPlaceWithFallback => {
                open_options.truncate(false);
            }
        }

        let file = match open_options.read(true).write(true).create(true).open(&path) {
            Ok(file) => file,
            Err(error) => {
                // Retry open operation with UnlinkAndReplace if it's an ETXTBSY error and
                // falllback is permitted.
                if error.kind() == ErrorKind::ExecutableFileBusy
                    && matches!(
                        options.file_replacement_mode,
                        FileReplacementMode::UpdateInPlaceWithFallback
                    )
                {
                    // If the file is being executed, we can't modify it, but we can delete it.
                    unlink_and_recreate(&path)?
                } else {
                    return Err(error)
                        .with_context(|| format!("Failed to open `{}`", path.display()));
                }
            }
        };

        let file =
            if options.file_replacement_mode == FileReplacementMode::UpdateInPlaceWithFallback {
                let metadata = file
                    .metadata()
                    .with_context(|| format!("Failed to stat output `{}`", path.display()))?;

                if metadata.is_file() && crate::host::fs::may_have_multiple_links(&metadata) {
                    drop(file);
                    unlink_and_recreate(&path)?
                } else {
                    file
                }
            } else {
                file
            };

        let defaults = OutputFileDefaults::for_file(&file);

        let fallocate = options.fallocate.unwrap_or(defaults.fallocate);
        let huge_pages_required = options.madvise_huge_pages == Some(true);
        let madvise_huge_pages = options
            .madvise_huge_pages
            .unwrap_or(defaults.madvise_huge_pages);
        let file_write_mode = options.write_mode.unwrap_or(defaults.write_mode);

        if huge_pages_required && matches!(file_write_mode, FileWriteMode::BufferThenWrite) {
            return Err(crate::error!(
                "--madvise-huge-pages requires mmapped output file"
            ));
        }

        let set_len_result = file.set_len(options.size);

        if fallocate
            && let Err(error) = crate::host::fs::preallocate(&file, options.size)
            && options.fallocate.is_some()
        {
            return Err(error).with_context(|| format!("Failed to fallocate `{}`", path.display()));
        }

        let buffer = match file_write_mode {
            FileWriteMode::Mmap => {
                // For some types of output file (e.g. character devices) we can't mmap, so we try
                // to mmap the file and if it fails, fall back to non-mmapped output.
                match set_len_result {
                    Ok(()) => match unsafe { MmapOptions::new().map_mut(&file) } {
                        Ok(mmap) => {
                            if let Err(error) =
                                advise_huge_pages_if_requested(&mmap, madvise_huge_pages)
                                && huge_pages_required
                            {
                                return Err(error).with_context(|| {
                                    format!("madvise huge pages failed for `{}`", path.display())
                                });
                            }
                            OsOutputBuffer::Mmap(mmap)
                        }
                        Err(error) if huge_pages_required => {
                            return Err(error).with_context(|| {
                                format!(
                                    "--madvise-huge-pages requires mmap, but mmap of `{}` failed",
                                    path.display()
                                )
                            });
                        }
                        Err(_) => OsOutputBuffer::InMemory(vec![0; options.size as usize]),
                    },
                    Err(error) if huge_pages_required => {
                        return Err(error).with_context(|| {
                            format!("Failed to set size `{}` for mmap", path.display())
                        });
                    }
                    Err(_) => OsOutputBuffer::InMemory(vec![0; options.size as usize]),
                }
            }
            FileWriteMode::BufferThenWrite => {
                // Try to set the length of the file. We ignore failures here because it's expected
                // to fail for some types of files, e.g. /dev/null. If there's actually a problem
                // writing to the file, we'll discover that when we go to write the content later
                // on.
                let _ = set_len_result;
                OsOutputBuffer::InMemory(vec![0; options.size as usize])
            }
        };

        Ok(OsOutputFile { file, buffer, path })
    }

    fn write_auxiliary(&self, path: &Path, bytes: &[u8]) -> Result {
        let file = File::create(path)?;
        (&file).write_all(bytes)?;
        Ok(())
    }
}

fn unlink_and_recreate(path: &Path) -> Result<File> {
    std::fs::remove_file(path)
        .with_context(|| format!("Failed to unlink output `{}`", path.display()))?;

    std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .open(path)
        .with_context(|| format!("Failed to recreate output `{}`", path.display()))
}

fn advise_huge_pages_if_requested(mmap: &memmap2::MmapMut, requested: bool) -> Result {
    if requested {
        crate::host::fs::advise_huge_pages(mmap)?;
    }
    Ok(())
}

struct OutputFileDefaults {
    write_mode: FileWriteMode,
    fallocate: bool,
    madvise_huge_pages: bool,
}

impl OutputFileDefaults {
    fn for_file(file: &std::fs::File) -> Self {
        let mut defaults = Self {
            write_mode: FileWriteMode::Mmap,
            fallocate: false,
            madvise_huge_pages: true,
        };

        match crate::host::fs::filesystem_kind(file) {
            // Multi-threaded write performance with BTRFS is terrible without huge pages and when
            // using huge pages with Linux < 7.2. It's substantially faster to just buffer it all in
            // memory then write it afterwards.
            Some(FilesystemKind::Btrfs)
                if crate::host::os::kernel_version().is_none_or(|version| version < (7, 2)) =>
            {
                defaults.write_mode = FileWriteMode::BufferThenWrite;
                defaults.madvise_huge_pages = false;
            }
            // vfat isn't quite as bad as BTRFS in this regard, but it's still at least 4-10% faster
            // if we avoid mmap.
            Some(FilesystemKind::Vfat) => {
                defaults.write_mode = FileWriteMode::BufferThenWrite;
            }
            Some(FilesystemKind::Ext4 | FilesystemKind::Xfs) => {
                defaults.fallocate = true;
            }
            Some(FilesystemKind::Btrfs | FilesystemKind::Other) | None => {}
        }

        defaults
    }
}

/// Make the the supplied file executable by adding execute permissions for all users that have read
/// permissions. On hosts without execute permissions, this is a no-op.
pub fn make_executable(file: &File) -> Result {
    crate::host::fs::make_executable(file)
}

pub(crate) fn path_from_bytes(bytes: &[u8]) -> PathBuf {
    crate::host::fs::path_from_bytes(bytes)
}
