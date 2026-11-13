use libwild::FileReplacementMode;
use libwild::FileSystem;
use libwild::FileType;
use libwild::OsFileSystem;
use libwild::OutputFileData;
use libwild::OutputOptions;
use libwild::ensure;
use libwild::error::Context as _;
use libwild::error::Result;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::ErrorKind;
use std::io::Write as _;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

/// An in-memory filesystem used by our integration tests when they use libwild directly unless the
/// test opts out. Input files are read from the real filesystem while the output files are stored
/// in memory and only written to disk at the end if there isn't already an identical file in place.
///
/// TODO: Remap paths so that paths used don't exist on the real file system.
#[derive(Clone, Default)]
pub(crate) struct TestFileSystem {
    outputs: Arc<Mutex<BTreeMap<PathBuf, BufferedFile>>>,
}

struct BufferedFile {
    bytes: Vec<u8>,
    executable: bool,
}

pub(crate) struct Output {
    path: PathBuf,
    bytes: Vec<u8>,
    file_system: TestFileSystem,
}

impl OutputFileData for Output {
    fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    fn bytes_mut(&mut self) -> &mut [u8] {
        &mut self.bytes
    }

    fn finish(self) -> Result {
        self.file_system.outputs.lock().unwrap().insert(
            self.path,
            BufferedFile {
                bytes: self.bytes,
                executable: true,
            },
        );
        Ok(())
    }
}

impl TestFileSystem {
    pub(crate) fn persist(self) -> Result {
        let outputs = Arc::try_unwrap(self.outputs)
            .map_err(|_| libwild::error!("Test filesystem is still in use"))?
            .into_inner()
            .unwrap();

        for (path, file) in outputs {
            persist_file(&path, &file)
                .with_context(|| format!("Failed to persist `{}`", path.display()))?;
        }

        Ok(())
    }
}

impl FileSystem for TestFileSystem {
    type Input = <OsFileSystem as FileSystem>::Input;
    type Output = Output;

    fn open_input(
        &self,
        path: &Path,
        prepopulate: bool,
    ) -> Result<(Self::Input, Option<Arc<File>>)> {
        OsFileSystem.open_input(path, prepopulate)
    }

    fn read_to_string(&self, path: &Path) -> Result<String> {
        OsFileSystem.read_to_string(path)
    }

    fn file_type(&self, path: &Path) -> Result<FileType> {
        OsFileSystem.file_type(path)
    }

    fn absolute_path(&self, path: &Path) -> Result<PathBuf> {
        OsFileSystem.absolute_path(path)
    }

    fn canonicalize(&self, path: &Path) -> Result<PathBuf> {
        OsFileSystem.canonicalize(path)
    }

    fn create_output(&self, path: Arc<Path>, options: OutputOptions) -> Result<Output> {
        ensure!(
            options.file_replacement_mode != FileReplacementMode::UpdateInPlace
                && options.write_mode.is_none()
                && options.fallocate.is_none()
                && options.madvise_huge_pages.is_none(),
            "OS output options require InMemoryFileSystem:false"
        );
        check_output_path(&path)?;
        let size = usize::try_from(options.size).context("Output is too large")?;
        Ok(Output {
            path: path.to_path_buf(),
            bytes: vec![0; size],
            file_system: self.clone(),
        })
    }

    fn write_auxiliary(&self, path: &Path, bytes: &[u8]) -> Result {
        check_output_path(path)?;
        self.outputs.lock().unwrap().insert(
            path.to_owned(),
            BufferedFile {
                bytes: bytes.to_vec(),
                executable: false,
            },
        );
        Ok(())
    }
}

fn check_output_path(path: &Path) -> Result {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) => ensure!(
            metadata.is_file(),
            "Not a regular output file: {}",
            path.display()
        ),
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    Ok(())
}

fn persist_file(path: &Path, file: &BufferedFile) -> Result {
    check_output_path(path)?;
    let unchanged = match std::fs::read(path) {
        Ok(bytes) => bytes == file.bytes,
        Err(error) if error.kind() == ErrorKind::NotFound => false,
        Err(error) => return Err(error.into()),
    };
    if unchanged {
        return Ok(());
    }

    if file.executable {
        match std::fs::remove_file(path) {
            Ok(()) => {}
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    let mut output = File::create(path)?;
    output.write_all(&file.bytes)?;
    if file.executable {
        libwild::make_executable(&output)?;
    }
    Ok(())
}
