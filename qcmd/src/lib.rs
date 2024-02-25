use std::{
    io::{Read, Write},
    process::{Command, Stdio},
};

/// A "quick" command that holds references to the source and sink.
///
/// The canonical way to construct this is with the `qshell::cmd!` macro.
pub struct QCmd<'source, 'sink> {
    cmd: Command,
    source: Option<&'source str>,
    sink: Option<&'sink mut String>,
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
    pub fn exec(mut self) -> Result<(), std::io::Error> {
        let mut child = self.cmd.spawn()?;
        if let Some(source) = self.source {
            // This can't panic since we set it up on construction.
            let mut stdin = child.stdin.take().unwrap();
            stdin.write_all(source.as_bytes())?;
        }
        // TODO: Make status checking optional.
        let status = child.wait().expect("Child process wasn't running?");
        assert!(status.success());
        if let Some(sink) = self.sink {
            // This can't panic since we set it up on construction.
            // TODO: Accept Vec<u8> as well as String.
            let mut stdout = child.stdout.take().unwrap();
            stdout.read_to_string(sink).expect("Non-UTF8 string");
        }
        Ok(())
    }
}
