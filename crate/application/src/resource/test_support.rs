use std::env;
use std::io::Write;
use std::path::PathBuf;

use tempdir::TempDir;
use tempfile::{NamedTempFile, NamedTempFileOptions};

// https://github.com/rust-lang/rust/issues/43155
/// Declares a `lazy_static` mutex with the name `TEST_MUTEX`.
#[macro_export]
macro_rules! test_mutex {
    () => {
        use std::panic;
        use std::sync::Mutex;

        lazy_static! {
            static ref TEST_MUTEX: Mutex<()> = Mutex::new(());
        }
    }
}

#[macro_export]
macro_rules! test {
    (fn $name:ident() $body:block) => {
        #[test]
        fn $name() {
            let guard = TEST_MUTEX.lock().unwrap();
            if let Err(e) = panic::catch_unwind(|| { $body }) {
                // kcov-ignore-start
                drop(guard);
                panic::resume_unwind(e);
                // kcov-ignore-end
            }
        }
    }
}

/// Returns the base directory path of the current executable.
pub(crate) fn exe_dir() -> PathBuf {
    let mut exe_dir = env::current_exe().unwrap();
    exe_dir.pop();
    exe_dir
}

/// Creates a temporary resource file in a directory for tests.
///
/// # Parameters
///
/// * `resource_dir`: Parent directory of the file. Either absolute, or relative to the executable.
/// * `file_prefix`: File stem, such as "display_config" in "display_config.ron".
/// * `file_suffix`: File extension including the ".", such as ".ron" in "display_config.ron".
/// * `contents`: String to write into the file.
pub(crate) fn setup_temp_file(
    resource_dir: &str,
    file_prefix: &str,
    file_suffix: &str,
    contents: Option<&str>,
) -> Option<(Option<TempDir>, NamedTempFile)> {
    let conf_path = PathBuf::from(resource_dir);

    // normalize relative paths to be relative to exe directory instead of working directory
    let exe_dir = exe_dir();
    let conf_parent;
    let temp_dir;

    // if the conf_path is absolute, or is the exe directory, we don't create a temp_dir
    if conf_path.is_absolute() || resource_dir == "" {
        conf_parent = exe_dir;
        temp_dir = None;
    } else {
        let tmp_dir = TempDir::new_in(exe_dir, resource_dir).unwrap();

        conf_parent = tmp_dir.path().to_owned();
        temp_dir = Some(tmp_dir);
    } // kcov-ignore

    let mut temp_file = NamedTempFileOptions::new()
                            .prefix(file_prefix)
                            .suffix(file_suffix)
                            // don't include randomly generated bytes in the file name
                            .rand_bytes(0)
                            .create_in(conf_parent)
                            .unwrap();

    if let Some(contents) = contents {
        write!(temp_file, "{}", contents).expect("Failed to write contents to temp_file");
    }

    return Some((temp_dir, temp_file));
}
