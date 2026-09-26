use colored::Colorize as _;
use std::fmt::Display;

pub type Result<T = (), E = Error> = core::result::Result<T, E>;

#[expect(clippy::box_collection)]
pub struct Error(Box<Vec<ErrorPayload>>);

struct ErrorPayload {
    messages: Vec<String>,
}

#[macro_export]
macro_rules! bail {
    ($msg:literal $(,)?) => {
        return Err($crate::error!($msg))
    };
    ($fmt:expr, $(,)?) => {
        return Err($crate::error!($expr))
    };
    ($fmt:expr, $($args:tt)*) => {
        return Err($crate::error!($fmt, $($args)*))
    };
}

#[macro_export]
macro_rules! error {
    ($msg:literal $(,)?) => {
        $crate::error::Error::with_message(format!($msg))
    };
    ($fmt:expr, $(,)?) => {
        $crate::error::Error::with_message(format!($fmt))
    };
    ($fmt:expr, $($args:tt)*) => {
        $crate::error::Error::with_message(format!($fmt, $($args)*))
    };
}

#[macro_export]
macro_rules! ensure {
    ($cond:expr, $msg:literal $(,)?) => {
        if !$cond {
            return Err($crate::error!($msg));
        }
    };
    ($cond:expr, $fmt:expr, $(,)?) => {
        if !$cond {
            return Err($crate::error!($expr));
        }
    };
    ($cond:expr, $fmt:expr, $($args:tt)*) => {
        if !$cond {
            return Err($crate::error!($fmt, $($args)*));
        }
    };
}

impl Error {
    pub fn with_message(msg: impl Into<String>) -> Self {
        Error(Box::new(vec![ErrorPayload {
            messages: vec![msg.into()],
        }]))
    }

    // We can't implement Display, since we implement From for things that are Display.
    #[allow(clippy::inherent_to_string)]
    #[must_use]
    pub fn to_string(&self) -> String {
        format!("{self:?}")
    }
}

/// An error indicating that we attempted to initialise global state that can only be initialised
/// once.
#[derive(Debug, Clone, Copy)]
pub struct AlreadyInitialised;

/// Like debug_assert, but bails instead of panicking.
///
/// Returning an error often allows us to give
/// more context as to what we were trying to do, e.g. which file / symbol we were processing,
/// whereas a panic just gives us a function backtrace, which is less useful.
#[macro_export]
macro_rules! debug_assert_bail {
    ($e:expr, $($rest:tt)*) => {
        if cfg!(debug_assertions) && !$e {
            $crate::bail!($($rest)*);
        }
    };
}

pub struct Warning {
    message: String,
}
impl Warning {
    #[must_use]
    pub fn new(message: String) -> Self {
        Self { message }
    }

    #[must_use]
    pub fn warning(&self) -> &str {
        &self.message
    }
}

impl Display for Warning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "wild: {} {}", "warning:".yellow(), self.message)
    }
}

impl Display for AlreadyInitialised {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Attempted to initialise global state more than once")
    }
}

impl core::error::Error for AlreadyInitialised {}

impl<E> From<E> for Error
where
    E: std::fmt::Display,
{
    fn from(value: E) -> Self {
        Error::with_message(format!("{value}"))
    }
}

impl Error {
    /// Convert an anyhow error, preserving the full error chain.
    #[must_use]
    #[allow(clippy::needless_pass_by_value)]
    pub fn from_anyhow(err: anyhow::Error) -> Self {
        let mut messages = vec![err.to_string()];
        for cause in err.chain().skip(1) {
            messages.push(cause.to_string());
        }
        Error(Box::new(vec![ErrorPayload { messages }]))
    }
}

pub trait Context<T> {
    fn with_context(self, callback: impl FnOnce() -> String) -> Result<T>;
    fn context(self, message: &'static str) -> Result<T>;
}

impl<T, E: Into<Error>> Context<T> for Result<T, E> {
    #[inline(always)]
    fn with_context(self, callback: impl FnOnce() -> String) -> Result<T> {
        match self {
            Ok(v) => Ok(v),
            Err(error) => Err(result_context_error(error, callback)),
        }
    }

    fn context(self, message: &'static str) -> Result<T> {
        match self {
            Ok(v) => Ok(v),
            Err(error) => {
                let mut error: Error = error.into();
                if let Some(last) = error.0.last_mut() {
                    last.messages.push(message.into());
                } else {
                    error.0.push(ErrorPayload {
                        messages: vec![message.into()],
                    });
                }
                Err(error)
            }
        }
    }
}

impl<T> Context<T> for Option<T> {
    #[inline(always)]
    fn with_context(self, callback: impl FnOnce() -> String) -> Result<T> {
        match self {
            Some(v) => Ok(v),
            None => Err(option_context_error(callback)),
        }
    }

    fn context(self, message: &'static str) -> Result<T> {
        match self {
            Some(v) => Ok(v),
            None => Err(Error::with_message(message)),
        }
    }
}

#[cold]
#[inline(never)]
fn result_context_error<E: Into<Error>>(error: E, callback: impl FnOnce() -> String) -> Error {
    let mut error: Error = error.into();
    if let Some(last) = error.0.last_mut() {
        last.messages.push(callback());
    } else {
        error.0.push(ErrorPayload {
            messages: vec![callback()],
        });
    }
    error
}

#[cold]
#[inline(never)]
fn option_context_error(callback: impl FnOnce() -> String) -> Error {
    Error::with_message(callback())
}

impl std::fmt::Debug for ErrorPayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.messages.len() == 1 {
            return writeln!(f, "{}", self.messages[0]);
        }

        let mut first = true;
        for message in self.messages.iter().rev() {
            if first {
                writeln!(f, "{message}")?;
                first = false;
                writeln!(f, "  Caused by:")?;
            } else {
                writeln!(f, "    {message}")?;
            }
        }
        Ok(())
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for payload in &*self.0 {
            payload.fmt(f)?;
        }
        Ok(())
    }
}

pub fn report_error(errors: &Error) {
    for error in &*errors.0 {
        eprint!("wild: {}: {error:?}", "error".red());
    }
}

pub fn report_error_and_exit(error: &Error) -> ! {
    report_error(error);
    std::process::exit(-1);
}

#[derive(Default)]
pub(crate) struct MultiErrorBuilder {
    errors: Vec<ErrorPayload>,
}

impl MultiErrorBuilder {
    pub(crate) fn new() -> Self {
        Self { errors: Vec::new() }
    }

    pub(crate) fn add_error(&mut self, error: Error) {
        self.errors.extend(*error.0);
    }

    pub(crate) fn emit_errors_if_any(self) -> Result {
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(Error(Box::new(self.errors)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chained_error(main: &'static str, context1: &'static str, context2: &'static str) -> Error {
        let mut result: Result = Err(Error::with_message(main));
        result = Err(result.context(context1).unwrap_err());
        result.context(context2).unwrap_err()
    }

    #[test]
    fn multiple_messages_without_causes_each_on_own_line() {
        let mut builder = MultiErrorBuilder::new();
        builder.add_error(Error::with_message("first"));
        builder.add_error(Error::with_message("second"));
        builder.add_error(Error::with_message("third"));
        let error = builder.emit_errors_if_any().unwrap_err();
        assert_eq!(error.to_string(), "first\nsecond\nthird\n");
    }

    #[test]
    fn multiple_messages_with_causes_each_on_own_line() {
        let mut builder = MultiErrorBuilder::new();
        builder.add_error(chained_error(
            "failed to load foo",
            "while linking",
            "first error",
        ));
        builder.add_error(chained_error(
            "failed to load bar",
            "while linking",
            "second error",
        ));
        builder.add_error(chained_error(
            "failed to load baz",
            "while linking",
            "third error",
        ));
        let error = builder.emit_errors_if_any().unwrap_err();
        assert_eq!(
            error.to_string(),
            "first error\n  Caused by:\n    while linking\n    failed to load foo\n\
            second error\n  Caused by:\n    while linking\n    failed to load bar\n\
            third error\n  Caused by:\n    while linking\n    failed to load baz\n"
        );
    }
}
