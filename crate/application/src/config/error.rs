use std::io;

use amethyst;
use amethyst::core;
use amethyst::config::ConfigError;
use error_chain;

use config::FindContext;

// kcov-ignore-start
/// `ErrorKind` for application configuration
#[derive(Debug, ErrorChain)]
pub enum ErrorKind {
    /// Plain error message without additional structure or context
    Msg(String),

    /// Error when failing to find a configuration file
    #[error_chain(custom, display = r#"|e| write!(f, "{}", e)"#)]
    Find(FindContext),

    /// Application configuration error due to an IO failure
    #[error_chain(foreign, display = r#"|e| write!(f, "io::Error: '{}'", e)"#)]
    Io(io::Error),
}
// kcov-ignore-end

impl From<FindContext> for Error {
    fn from(find_context: FindContext) -> Error {
        Error(ErrorKind::Find(find_context), error_chain::State::default())
    }
}

impl From<Error> for io::Error {
    fn from(config_error: Error) -> io::Error {
        match config_error.0 /* error_kind */ {
            ErrorKind::Msg(msg) => io::Error::new(io::ErrorKind::Other, msg),
            ErrorKind::Find(find_context) => {
                io::Error::new(io::ErrorKind::Other, format!("{}", find_context))
            }
            ErrorKind::Io(io_error) => io_error,
        }
    }
}

impl From<Error> for amethyst::Error {
    fn from(config_error: Error) -> amethyst::Error {
        let config_error = ConfigError::File(config_error.into());
        amethyst::Error::Config(config_error)
    }
}

impl From<Error> for core::Error {
    fn from(config_error: Error) -> core::Error {
        core::Error::from_kind(core::ErrorKind::Msg(format!("{}", config_error)))
    }
}
