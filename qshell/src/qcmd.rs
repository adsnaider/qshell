//! Command-wrapper that simplifies piping and other operations.
use std::{
    io::{Read, Write},
    process::{Command, Stdio},
};

use thiserror::Error;

/// Builder object for [`QCmd`]
pub struct QCmdBuilder<'source, 'sink> {
    cmd: Command,
    source: Option<&'source str>,
    sink: Option<&'sink mut String>,
}

impl<'source, 'sink> QCmdBuilder<'source, 'sink> {
    /// Creates the builder with the given Command.
    pub fn new(cmd: Command) -> Self {
        Self {
            cmd,
            source: None,
            sink: None,
        }
    }

    /// Sets the source
    pub fn source(mut self, source: &'source str) -> Self {
        self.source.replace(source);
        self
    }

    /// Sets the sink
    pub fn sink(mut self, sink: &'sink mut String) -> Self {
        self.sink.replace(sink);
        self
    }

    /// Builds the [`QCmd`], consuming the builder.
    pub fn build(self) -> QCmd<'source, 'sink> {
        QCmd::new(self.cmd, self.source, self.sink)
    }
}

/// A "quick" command that holds references to the source and sink.
///
/// The canonical way to construct this is with the `qshell::cmd!` macro.
pub struct QCmd<'source, 'sink> {
    cmd: Command,
    source: Option<&'source str>,
    sink: Option<&'sink mut String>,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO Error while executing the command")]
    Io(#[from] std::io::Error),
    #[error("Command returned a non okay status")]
    StatusFailure(i32),
    #[error("Process was terminated by a signal")]
    UnexpectedTermination,
}

impl<'source, 'sink> QCmd<'source, 'sink> {
    /// Creates a new QCmd with the optional source and sink.
    pub fn new(
        mut cmd: Command,
        source: Option<&'source str>,
        mut sink: Option<&'sink mut String>,
    ) -> Self {
        if source.is_some() {
            cmd.stdin(Stdio::piped());
        }
        if let Some(sink) = sink.as_mut() {
            sink.clear();
            cmd.stdout(Stdio::piped());
        }
        cmd.stdin(Stdio::piped());
        Self { cmd, source, sink }
    }

    /// Executes the command piping the I/O as requested.
    pub fn exec(mut self) -> Result<(), Error> {
        let mut child = self.cmd.spawn()?;
        if let Some(source) = self.source {
            // This can't panic since we set it up on construction.
            let mut stdin = child.stdin.take().unwrap();
            stdin.write_all(source.as_bytes())?;
        }
        let status = child.wait().expect("Child process wasn't running?");
        match status.code() {
            Some(i) if i != 0 => return Err(Error::StatusFailure(i)),
            Some(zero) => debug_assert!(zero == 0),
            None => return Err(Error::UnexpectedTermination),
        }
        if let Some(sink) = self.sink {
            // This can't panic since we set it up on construction.
            // TODO: Accept Vec<u8> as well as String.
            let mut stdout = child.stdout.take().unwrap();
            stdout.read_to_string(sink).expect("Non-UTF8 string");
        }
        Ok(())
    }
}
