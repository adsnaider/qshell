//! Command-wrapper that simplifies piping and other operations.
use std::{
    borrow::Cow,
    io::{ErrorKind, Read, Write},
    process::{Command, Stdio},
};

use thiserror::Error;

/// Builder object for [`QCmd`]
#[derive(Debug)]
pub struct QCmdBuilder<'source, 'sink> {
    cmd: Command,
    source: Source<'source>,
    sink: Sink<'sink>,
}

/// Command stdin sources.
#[derive(Debug, Default)]
pub enum Source<'source> {
    /// Inherit the stdin pipe
    #[default]
    Stdin,
    /// Pipe the input from a string-like object
    Str(Cow<'source, str>),
    /// Pipe the input from a bytes-like object
    Bytes(Cow<'source, [u8]>),
}

/// Command stdout sinks
#[derive(Debug, Default)]
pub enum Sink<'sink> {
    /// Inherit the stdout pipe
    #[default]
    Stdout,
    /// Append output to a [`String`]
    Str(&'sink mut String),
    /// Append output to a [`Vec<u8>`]
    Bytes(&'sink mut Vec<u8>),
}

impl<'source, 'sink> QCmdBuilder<'source, 'sink> {
    /// Creates the builder with the given Command.
    pub fn new(cmd: Command) -> Self {
        Self {
            cmd,
            source: Source::Stdin,
            sink: Sink::Stdout,
        }
    }

    /// Sets the source
    pub fn source(&mut self, source: impl Into<Source<'source>>) -> &mut Self {
        self.source = source.into();
        self
    }

    /// Sets the sink
    pub fn sink(&mut self, sink: impl Into<Sink<'sink>>) -> &mut Self {
        self.sink = sink.into();
        self
    }

    /// Builds the [`QCmd`], consuming the builder.
    pub fn build(self) -> QCmd<'source, 'sink> {
        QCmd::new(self.cmd, self.source, self.sink)
    }
}

/// A "quick" command that holds references to the source and sink.
///
/// The canonical way to construct this is with the [`cmd!`](crate::cmd) macro.
#[derive(Debug)]
pub struct QCmd<'source, 'sink> {
    cmd: Command,
    source: Source<'source>,
    sink: Sink<'sink>,
}

/// Error executing the command
#[derive(Error, Debug)]
pub enum Error {
    /// IO Error
    #[error("IO Error while executing the command")]
    Io(#[from] std::io::Error),
    /// Non-zero status code
    #[error("Command returned a non okay status")]
    StatusFailure(i32),
    /// Unexpected termination by signal
    #[error("Process was terminated by a signal")]
    UnexpectedTermination,
    /// Output data is not UTF8
    #[error("Piped output does not conform to UTF-8")]
    NotUtf8,
}

impl<'source, 'sink> QCmd<'source, 'sink> {
    /// Creates a new QCmd with the optional source and sink.
    pub fn new(mut cmd: Command, source: Source<'source>, sink: Sink<'sink>) -> Self {
        match &source {
            Source::Stdin => cmd.stdin(Stdio::inherit()),
            Source::Str(_) | Source::Bytes(_) => cmd.stdin(Stdio::piped()),
        };
        match &sink {
            Sink::Stdout => cmd.stdout(Stdio::inherit()),
            Sink::Str(_) | Sink::Bytes(_) => cmd.stdout(Stdio::piped()),
        };
        Self { cmd, source, sink }
    }

    /// Executes the command piping the I/O as requested.
    pub fn exec(mut self) -> Result<(), Error> {
        let mut child = self.cmd.spawn()?;
        match self.source {
            Source::Stdin => {}
            Source::Str(source) => {
                let mut stdin = child.stdin.take().unwrap();
                stdin.write_all(source.as_bytes())?;
            }
            Source::Bytes(source) => {
                let mut stdin = child.stdin.take().unwrap();
                stdin.write_all(source.as_ref())?;
            }
        }
        let status = child.wait().expect("Child process wasn't running?");
        match status.code() {
            Some(i) if i != 0 => return Err(Error::StatusFailure(i)),
            Some(zero) => debug_assert!(zero == 0),
            None => return Err(Error::UnexpectedTermination),
        }
        match self.sink {
            Sink::Stdout => {}
            Sink::Str(sink) => {
                // This can't fail due to QCmd::new
                let mut stdout = child.stdout.take().unwrap();
                if let Err(e) = stdout.read_to_string(sink) {
                    match e.kind() {
                        ErrorKind::InvalidData => return Err(Error::NotUtf8),
                        _ => return Err(Error::Io(e)),
                    }
                }
            }
            Sink::Bytes(sink) => {
                // This can't fail due to QCmd::new
                let mut stdout = child.stdout.take().unwrap();
                stdout.read_to_end(sink)?;
            }
        }
        Ok(())
    }
}

impl<'source> From<&'source str> for Source<'source> {
    fn from(value: &'source str) -> Self {
        Self::Str(Cow::Borrowed(value))
    }
}
impl<'source> From<String> for Source<'source> {
    fn from(value: String) -> Self {
        Self::Str(Cow::Owned(value))
    }
}
impl<'source> From<&'source [u8]> for Source<'source> {
    fn from(value: &'source [u8]) -> Self {
        Self::Bytes(Cow::Borrowed(value))
    }
}
impl<'source> From<Vec<u8>> for Source<'source> {
    fn from(value: Vec<u8>) -> Self {
        Self::Bytes(Cow::Owned(value))
    }
}

impl<'sink> From<&'sink mut String> for Sink<'sink> {
    fn from(value: &'sink mut String) -> Self {
        Self::Str(value)
    }
}
impl<'sink> From<&'sink mut Vec<u8>> for Sink<'sink> {
    fn from(value: &'sink mut Vec<u8>) -> Self {
        Self::Bytes(value)
    }
}
