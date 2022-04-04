//! Module exposing APIs based around `AsyncBufRead` from Tokio.
use futures::stream::{self, Stream};
use tokio::io::{AsyncBufRead, AsyncBufReadExt};

use std::io::Error;

/// Provides async iteration over bytes of input, split by line.
///
/// ```rust ignore
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
/// more closely with the Tokio API for types.
///
/// For those who prefer the `Stream` API, this structure can be
/// converted using `into_stream`. This comes at the cost of an
/// allocation of a `Vec` for each line in the `Stream`. This is
/// negligible in many cases, so often it comes down to which
/// syntax is preferred:
///
/// ```rust ignore
/// use bytelines::*;
/// use std::fs::File;
/// use std::io::BufReader;
///
/// // construct our iterator from our file input
/// let file = File::open("./res/numbers.txt").await?;
/// let reader = BufReader::new(file);
/// let mut lines = AsyncByteLines::new(reader);
///
/// // walk our lines using `Stream` syntax
/// lines.into_stream().for_each(|line| {
///
/// });
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

    /// Converts this wrapper to provide a `Stream` API.
    pub fn into_stream(self) -> impl Stream<Item = Result<Vec<u8>, Error>> {
        stream::try_unfold(self, |mut lines| async {
            Ok(lines
                .next()
                .await?
                .map(|line| line.to_vec())
                .map(|line| (line, lines)))
        })
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

    #[tokio::test]
    async fn test_basic_stream() {
        use futures::StreamExt;

        let file = File::open("./res/numbers.txt").await.unwrap();
        let brdr = BufReader::new(file);

        let lines = crate::from_tokio(brdr)
            .into_stream()
            .map(|line| String::from_utf8(line.unwrap()).unwrap())
            .collect::<Vec<_>>()
            .await;

        for i in 0..9 {
            assert_eq!(lines[i], format!("{}", i));
        }
    }
}
