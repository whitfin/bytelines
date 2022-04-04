//! Module exposing APIs based around `AsyncBufRead` from Tokio.
use std::io::Error;
use tokio::io::{AsyncBufRead, AsyncBufReadExt};

/// Provides async iteration over bytes of input, split by line.
///
/// ```rust
/// use bytelines::*;
/// use std::fs::File;
/// use std::io::BufReader;
///
/// // construct our iterator from our file input
/// let file = File::open("./res/numbers.txt").await?;
/// let reader = BufReader::new(file);
/// let mut lines = AsyncByteLines::new(reader);
///
/// // walk our lines using `while` syntax
/// while let Some(line) = lines.next().await? {
///     // do something with the line, which is &[u8]
/// }
///
/// This differs from the `stdlib` version of the API as it fits
/// more closely with the Tokio API for types. There is no current
/// `Stream` based API for this form as of yet.
/// ```
pub struct AsyncByteLines<B>
where
    B: AsyncBufRead + Unpin,
{
    buffer: Vec<u8>,
    reader: B,
}

impl<B> AsyncByteLines<B>
where
    B: AsyncBufRead + Unpin,
{
    /// Constructs a new `ByteLines` from an input `AsyncBufRead`.
    pub fn new(buf: B) -> Self {
        Self {
            buffer: Vec::new(),
            reader: buf,
        }
    }

    /// Retrieves a reference to the next line of bytes in the reader (if any).
    pub async fn next(&mut self) -> Result<Option<&[u8]>, Error> {
        self.buffer.clear();
        let handled = crate::util::handle_line(
            self.reader.read_until(b'\n', &mut self.buffer).await,
            &mut self.buffer,
        );
        handled.transpose()
    }
}

#[cfg(test)]
#[allow(clippy::needless_range_loop)]
mod tests {
    use tokio::fs::File;
    use tokio::io::BufReader;

    #[tokio::test]
    async fn test_basic_loop() {
        let file = File::open("./res/numbers.txt").await.unwrap();
        let brdr = BufReader::new(file);
        let mut brdr = crate::from_tokio(brdr);
        let mut lines = Vec::new();

        while let Some(line) = brdr.next().await.unwrap() {
            let line = line.to_vec();
            let line = String::from_utf8(line).unwrap();

            lines.push(line);
        }

        for i in 0..9 {
            assert_eq!(lines[i], format!("{}", i));
        }
    }
}
