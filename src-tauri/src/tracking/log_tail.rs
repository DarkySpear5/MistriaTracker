use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::{Path, PathBuf},
};

const MAX_PENDING_BYTES: usize = 256 * 1024;

#[derive(Debug, thiserror::Error)]
pub enum LogTailError {
    #[error("could not read companion log: {0}")]
    Io(#[from] std::io::Error),
    #[error("new companion log data exceeds the read-only safety limit")]
    TooLarge,
    #[error("companion log contains invalid UTF-8")]
    InvalidUtf8,
}

/// A bounded, read-only cursor over newly appended complete log lines.
pub struct ReadOnlyLogTail {
    path: PathBuf,
    offset: u64,
    pending: Vec<u8>,
}

impl ReadOnlyLogTail {
    /// Starts at the end of an existing log so old history is never replayed.
    pub fn open(path: &Path) -> Result<Self, LogTailError> {
        Ok(Self {
            path: path.to_path_buf(),
            offset: std::fs::metadata(path)?.len(),
            pending: Vec::new(),
        })
    }

    pub fn poll(&mut self) -> Result<Vec<String>, LogTailError> {
        let length = std::fs::metadata(&self.path)?.len();
        if length < self.offset {
            self.offset = 0;
            self.pending.clear();
        }
        let unread = length - self.offset;
        if unread > MAX_PENDING_BYTES as u64
            || self.pending.len() + unread as usize > MAX_PENDING_BYTES
        {
            return Err(LogTailError::TooLarge);
        }
        if unread == 0 {
            return Ok(Vec::new());
        }

        let mut file = File::open(&self.path)?;
        file.seek(SeekFrom::Start(self.offset))?;
        let mut bytes = vec![0; unread as usize];
        file.read_exact(&mut bytes)?;
        self.offset = length;
        self.pending.extend(bytes);

        let mut lines = Vec::new();
        let mut start = 0;
        for index in 0..self.pending.len() {
            if self.pending[index] == b'\n' {
                let line = &self.pending[start..index];
                let line = std::str::from_utf8(line).map_err(|_| LogTailError::InvalidUtf8)?;
                lines.push(line.trim_end_matches('\r').to_owned());
                start = index + 1;
            }
        }
        self.pending.drain(..start);
        Ok(lines)
    }
}
