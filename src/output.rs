use std::io::{BufWriter, ErrorKind, Write};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OutputError {
    BrokenPipe,

    Other,
}

impl OutputError {
    fn classify(error: &std::io::Error) -> Self {
        if error.kind() == ErrorKind::BrokenPipe {
            Self::BrokenPipe
        } else {
            Self::Other
        }
    }
}

#[derive(Debug)]
pub(crate) struct Output<W: Write> {
    writer: BufWriter<W>,
}

impl<W: Write> Output<W> {
    pub(crate) fn new(sink: W) -> Self {
        Self {
            writer: BufWriter::new(sink),
        }
    }

    pub(crate) fn write_all(&mut self, bytes: &[u8]) -> Result<(), OutputError> {
        self.writer
            .write_all(bytes)
            .map_err(|error| OutputError::classify(&error))
    }

    pub(crate) fn flush(&mut self) -> Result<(), OutputError> {
        self.writer
            .flush()
            .map_err(|error| OutputError::classify(&error))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[derive(Debug)]
    struct FailingWriter {
        kind: ErrorKind,
    }

    impl Write for FailingWriter {
        fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
            Err(io::Error::new(self.kind, "simulated failure"))
        }

        fn flush(&mut self) -> io::Result<()> {
            Err(io::Error::new(self.kind, "simulated failure"))
        }
    }

    #[test]
    fn writes_bytes_verbatim_including_nul() {
        let mut sink = Vec::new();
        {
            let mut output = Output::new(&mut sink);
            output.write_all(b"a\n").expect("write");
            output.write_all(&[0x00, 0xFF]).expect("write");
            output.flush().expect("flush");
        }
        assert_eq!(sink, b"a\n\x00\xFF");
    }

    #[test]
    fn broken_pipe_is_classified_as_broken_pipe() {
        let mut output = Output::new(FailingWriter {
            kind: ErrorKind::BrokenPipe,
        });

        assert_eq!(output.flush(), Err(OutputError::BrokenPipe));
    }

    #[test]
    fn other_failures_are_classified_as_other() {
        let mut output = Output::new(FailingWriter {
            kind: ErrorKind::PermissionDenied,
        });
        assert_eq!(output.flush(), Err(OutputError::Other));
    }

    #[test]
    fn a_large_write_surfaces_the_failure_immediately() {
        let mut output = Output::new(FailingWriter {
            kind: ErrorKind::PermissionDenied,
        });

        let large = vec![0u8; 64 * 1024];
        assert_eq!(output.write_all(&large), Err(OutputError::Other));
    }
}
