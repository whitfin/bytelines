//! `Bytelines` is a simple library crate which offers line iteration for
//! `BufRead` via `&[u8]` rather than `String`.
//!
//! Due to the removal of checking for `String` validity, this is typically
//! much faster for reading in raw data and much more flexible. The APIs
//! offered in this crate are intended to function exactly the same as the
//! `lines` function inside the `BufRead` trait, except that the bytes which
//! precede the line delimiter are not validated.
//!
//! Performance of [ByteLines](enum.ByteLines.html) is practically identical
//! to that of writing a `loop` manually, due to the avoidance of allocations.
#![doc(html_root_url = "https://docs.rs/bytelines/2.2.1")]
use std::io::BufRead;

/// Represents anything which can provide iterators of byte lines.
pub trait ByteLinesReader<B>
where
    B: BufRead,
{
    /// Returns an structure used to iterate the lines of this reader as `&[u8]`.
    fn byte_lines(self) -> ByteLines<B>;

    /// Returns an iterator over the lines of this reader as `Vec<u8>`.
    fn byte_lines_iter(self) -> ByteLinesIter<B>;
}

/// Blanket implementation for all `BufRead`.
impl<B> ByteLinesReader<B> for B
where
    B: BufRead,
{
    /// Returns an structure used to iterate the lines of this reader as &[u8].
    #[inline]
    fn byte_lines(self) -> ByteLines<Self> {
        ByteLines {
            buffer: Vec::new(),
            reader: self,
        }
    }

    /// Returns an iterator over the lines of this reader (as `Vec<u8>`).
    #[inline]
    fn byte_lines_iter(self) -> ByteLinesIter<Self> {
        self.byte_lines().into_iter()
    }
}

/// Provides iteration over bytes of input, split by line.
///
/// Unlike the implementation in the standard library, this requires
/// no allocations and simply references the input lines from the
/// internal buffer. In order to do this safely, we must sacrifice
/// the `Iterator` API, and operate using `while` syntax:
///
/// ```rust
/// use bytelines::*;
/// use std::fs::File;
/// use std::io::BufReader;
///
/// // construct our iterator from our file input
/// let file = File::open("./res/numbers.txt").unwrap();
/// let mut lines = BufReader::new(file).byte_lines();
///
/// // walk our lines using `while` syntax
/// while let Some(line) = lines.next() {
///     // do something with the line, which is &[u8]
/// }
/// ```
///
/// For those who prefer the `Iterator` API, this structure implements
/// the `IntoIterator` trait to provide it. This comes at the cost of
/// an allocation of a `Vec` for each line in the `Iterator`. This is
/// negligible in many cases, so often it comes down to which syntax
/// is preferred:
///
/// ```rust
/// use bytelines::*;
/// use std::fs::File;
/// use std::io::BufReader;
///
/// // construct our iterator from our file input
/// let file = File::open("./res/numbers.txt").unwrap();
/// let lines = BufReader::new(file).byte_lines();
///
/// // walk our lines using `for` syntax
/// for line in lines.into_iter() {
///     // do something with the line, which is Vec<u8>
/// }
/// ```
pub struct ByteLines<B>
where
    B: BufRead,
{
    buffer: Vec<u8>,
    reader: B,
}

impl<B> ByteLines<B>
where
    B: BufRead,
{
    /// Constructs a new `ByteLines` from an input `BufRead`.
    pub fn new(buf: B) -> Self {
        Self {
            buffer: Vec::new(),
            reader: buf,
        }
    }

    /// Retrieves a reference to the next line of bytes in the reader (if any).
    pub fn next(&mut self) -> Option<Result<&[u8], std::io::Error>> {
        // clear the main buffer
        self.buffer.clear();

        // iterate every line coming from the reader (but as bytes)
        match self.reader.read_until(b'\n', &mut self.buffer) {
            // short circuit on error
            Err(e) => Some(Err(e)),

            // no input, done
            Ok(0) => None,

            // bytes!
            Ok(mut n) => {
                // always "pop" the delim
                if self.buffer[n - 1] == b'\n' {
                    n -= 1;
                    // also "pop" a leading \r
                    if self.buffer[n - 1] == b'\r' {
                        n -= 1;
                    }
                }

                // pass back the byte slice
                Some(Ok(&self.buffer[..n]))
            }
        }
    }
}

/// `IntoIterator` conversion for `ByteLines` to provide `Iterator` APIs.
impl<B> IntoIterator for ByteLines<B>
where
    B: BufRead,
{
    type Item = Result<Vec<u8>, std::io::Error>;
    type IntoIter = ByteLinesIter<B>;

    /// Constructs an `ByteLinesIter` to provide an `Iterator` API.
    #[inline]
    fn into_iter(self) -> ByteLinesIter<B> {
        ByteLinesIter { inner: self }
    }
}

/// `Iterator` implementation of `ByteLines` to provide `Iterator` APIs.
///
/// This structure enables developers the use of the `Iterator` API in
/// their code, at the cost of an allocation per input line:
///
/// ```rust
/// use bytelines::*;
/// use std::fs::File;
/// use std::io::BufReader;
///
/// // construct our iterator from our file input
/// let file = File::open("./res/numbers.txt").unwrap();
/// let lines = BufReader::new(file).byte_lines();
///
/// // walk our lines using `for` syntax
/// for line in lines.into_iter() {
///     // do something with the line, which is Vec<u8>
/// }
/// ```
pub struct ByteLinesIter<B>
where
    B: BufRead,
{
    inner: ByteLines<B>,
}

impl<B> Iterator for ByteLinesIter<B>
where
    B: BufRead,
{
    type Item = Result<Vec<u8>, std::io::Error>;

    /// Retrieves the next line in the iterator (if any).
    #[inline]
    fn next(&mut self) -> Option<Result<Vec<u8>, std::io::Error>> {
        self.inner.next().map(|r| r.map(|s| s.to_vec()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::BufReader;

    #[test]
    fn test_basic_loop() {
        let file = File::open("./res/numbers.txt").unwrap();
        let mut brdr = BufReader::new(file).byte_lines();
        let mut lines = Vec::new();

        while let Some(line) = brdr.next() {
            let line = line.unwrap().to_vec();
            let line = String::from_utf8(line).unwrap();

            lines.push(line);
        }

        for i in 0..9 {
            assert_eq!(lines[i], format!("{}", i));
        }
    }

    #[test]
    fn test_basic_iterator() {
        let file = File::open("./res/numbers.txt").unwrap();
        let mut lines = Vec::new();

        for line in BufReader::new(file).byte_lines().into_iter() {
            let line = line.unwrap();
            let line = String::from_utf8(line).unwrap();

            lines.push(line);
        }

        for i in 0..9 {
            assert_eq!(lines[i], format!("{}", i));
        }
    }
}
