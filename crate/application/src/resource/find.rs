use std::env;
use std::ffi;
use std::path::{Path, PathBuf};

use resource::error::Result;
use resource::FindContext;

/// Returns development-time base directories as a `Vec<::std::path::Path>`.
///
/// Currently this includes the following directories:
///
/// * [`option_env!`][1]`("`[`OUT_DIR`][2]`")`
/// * [`option_env!`][1]`("`[`CARGO_MANIFEST_DIR`][2]`")`
///
/// This has to be invoked by the consumer crate as the environmental variables change based on
/// which crate invokes the macro. This cannot be a function as the environmental variables are
/// evaluated at compile time of this crate.
///
/// [1]: https://doc.rust-lang.org/std/macro.option_env.html
/// [2]: http://doc.crates.io/environment-variables.html#environment-variables-cargo-sets-for-crates
#[macro_export]
macro_rules! development_base_dirs {
    () => {
        vec![option_env!("OUT_DIR"), option_env!("CARGO_MANIFEST_DIR")]
            .iter()
            .filter(|dir| dir.is_some())
            .map(|dir| dir.expect("Unwrapping option"))
            .map(|dir| ::std::path::Path::new(&dir).to_owned())
            .collect()
    }
}

/// Finds and returns the path to the configuration file.
///
/// # Parameters:
///
/// * `file_name`: Name of the file to search for which should be next to the executable.
pub fn find(file_name: &str) -> Result<PathBuf> {
    find_in(Path::new(""), file_name, None)
}

/// Finds and returns the path to the configuration file within the given configuration directory.
///
/// # Parameters:
///
/// * `conf_dir`: Directory relative to the executable in which to search for configuration.
/// * `file_name`: Name of the file to search for.
/// * `additional_base_dirs`: Additional base directories to look into. Useful at development time
///     when configuration is generated and placed in a separate output directory.
///
///     When compiled as `#[cfg(test)]`, `development_base_dirs!()` are automatically appended to
///     the base directories to search in.
pub fn find_in<P: AsRef<Path> + AsRef<ffi::OsStr>>(
    conf_dir: P,
    file_name: &str,
    additional_base_dirs: Option<Vec<PathBuf>>,
) -> Result<PathBuf> {
    let mut exe_dir = env::current_exe()?;
    exe_dir.pop();

    let mut base_dirs = vec![exe_dir];

    if let Some(mut additional_dirs) = additional_base_dirs {
        base_dirs.append(&mut additional_dirs);
    }

    if cfg!(debug_assertions) {
        base_dirs.push(development_base_dirs!());
    }

    for base_dir in &base_dirs {
        let mut resource_path = base_dir.join(&conf_dir);
        resource_path.push(&file_name);

        if resource_path.exists() {
            return Ok(resource_path);
        }
    }

    let find_context = FindContext {
        base_dirs,
        conf_dir: PathBuf::from(&conf_dir),
        file_name: file_name.to_owned(),
    }; // kcov-ignore
    Err(find_context.into())
}

/// The tests in here rely on file system state, which can cause failures when one test creates a
/// temporary file, and another test expects an Error when the file does not exist (but it does).
///
/// This is mentioned in the following issues:
///
/// * https://github.com/rust-lang/rust/issues/33519
/// * https://github.com/rust-lang/rust/pull/42684#issuecomment-314224230
/// * https://github.com/rust-lang/rust/issues/43155
///
/// We use a static mutex to ensure these tests are run serially. The code is taken from the third
/// link above.
#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use resource::dir;
    use resource::error::ErrorKind;
    use resource::FindContext;
    use resource::test_support::{exe_dir, setup_temp_file};
    use super::{find, find_in};

    test_mutex!();

    test! {
        fn find_in_returns_resource_path_when_file_exists() {
            let (temp_dir, resource_path) =
                setup_temp_file(dir::RESOURCES, "test__find_config", ".ron", None).unwrap();
            let temp_dir = temp_dir.unwrap();

            let expected = temp_dir.path().join("test__find_config.ron");
            assert_eq!(
                expected,
                find_in(
                    &temp_dir.path(),
                    "test__find_config.ron",
                    Some(development_base_dirs!())
                ).unwrap()
            );

            resource_path.close().unwrap();
            temp_dir.close().unwrap();
        }
    }

    test! {
        fn find_returns_resource_path_when_file_exists() {
            let (_, resource_path) =
                setup_temp_file("", "test__find_config", ".ron", None).unwrap();

            assert_eq!(
                exe_dir().join("test__find_config.ron"),
                find("test__find_config.ron").unwrap()
            );

            resource_path.close().unwrap();
        }
    }

    test! {
        fn find_returns_error_when_file_does_not_exist() {
            // We don't setup_temp_file(..);

            if let &ErrorKind::Find(ref find_context) =
                find("test__find_config.ron").unwrap_err().kind()
            {
                let mut base_dirs = vec![exe_dir()];
                base_dirs.append(&mut development_base_dirs!());
                let expected = FindContext {
                    base_dirs,
                    conf_dir: PathBuf::from(""),
                    file_name: "test__find_config.ron".to_owned(),
                }; // kcov-ignore

                assert_eq!(&expected, find_context);
            } else {
                panic!("Expected `find` to return error"); // kcov-ignore
            }
        }
    }

    test! {
        fn find_in_returns_error_when_file_does_not_exist() {
            // We don't setup_temp_file(..);

            let find_result = find_in(
                "",
                "test__find_config.ron",
                None,
            );

            if let &ErrorKind::Find(ref find_context) = find_result.unwrap_err().kind() {
                let mut base_dirs = vec![exe_dir()];
                base_dirs.append(&mut development_base_dirs!());
                let expected = FindContext {
                    base_dirs,
                    conf_dir: PathBuf::from(""),
                    file_name: "test__find_config.ron".to_owned(),
                }; // kcov-ignore

                assert_eq!(&expected, find_context);
            } else {
                panic!("Expected `find_in` to return error"); // kcov-ignore
            }
        }
    }

}
