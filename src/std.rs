//! Module exposing APIs based around `BufRead` from stdlib.
use hrtb_lending_iterator::*;
use std::io::{BufRead, Error};

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
/// use hrtb_lending_iterator::*;
///
/// // construct our iterator from our file input
/// let file = File::open("./res/numbers.txt").unwrap();
/// let reader = BufReader::new(file);
/// let mut lines = ByteLines::new(reader);
///
/// // walk our lines using `while` syntax
/// while let Some(line) = lines.next() {
///     // do something with the line, which is Result<&[u8], _>
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
/// let reader = BufReader::new(file);
/// let mut lines = ByteLines::new(reader);
///
/// // walk our lines using `for` syntax
/// for line in lines.into_iter() {
///     // do something with the line, which is Result<Vec<u8>, _>
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
}

/// `IntoIterator` conversion for `ByteLines` to provide `Iterator` APIs.
impl<B> IntoIterator for ByteLines<B>
where
    B: BufRead,
{
    type Item = Result<Vec<u8>, Error>;
    type IntoIter = ByteLinesIter<B>;

    /// Constructs a `ByteLinesIter` to provide an `Iterator` API.
    #[inline]
    fn into_iter(self) -> ByteLinesIter<B> {
        ByteLinesIter { inner: self }
    }
}

impl<'a, B: BufRead> LendingIteratorItem<'a> for ByteLines<B> {
    type Type = Result<&'a [u8], Error>;
}

impl<B: BufRead> LendingIterator for ByteLines<B> {
    /// Retrieves a reference to the next line of bytes in the reader (if any).
    fn next(&mut self) -> Option<Item<'_, Self>> {
        self.buffer.clear();
        crate::util::handle_line(
            self.reader.read_until(b'\n', &mut self.buffer),
            &mut self.buffer,
        )
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
/// let lines = BufReader::new(file);
/// let lines = bytelines::from_std(lines);
///
/// // walk our lines using `for` syntax
/// for line in lines.into_iter() {
///     // do something with the line, which is Result<Vec<u8>, _>
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
    type Item = Result<Vec<u8>, Error>;

    /// Retrieves the next line in the iterator (if any).
    #[inline]
    fn next(&mut self) -> Option<Result<Vec<u8>, Error>> {
        self.inner.next().map(|r| r.map(|s| s.to_vec()))
    }
}

/// Represents anything which can provide iterators of byte lines.
pub trait ByteLinesReader<B>
where
    B: BufRead,
{
    /// Returns a structure used to iterate the lines of this reader as `Result<&[u8], _>`.
    fn byte_lines(self) -> ByteLines<B>;
}

/// Blanket implementation for all `BufRead`.
impl<B> ByteLinesReader<B> for B
where
    B: BufRead,
{
    /// Returns a structure used to iterate the lines of this reader as Result<&[u8], _>.
    #[inline]
    fn byte_lines(self) -> ByteLines<Self> {
        super::from_std(self)
    }
}

#[cfg(test)]
#[allow(clippy::needless_range_loop)]
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

        for line in IntoIterator::into_iter(BufReader::new(file).byte_lines()) {
            let line = line.unwrap();
            let line = String::from_utf8(line).unwrap();

            lines.push(line);
        }

        for i in 0..9 {
            assert_eq!(lines[i], format!("{}", i));
        }
    }

    #[test]
    fn test_empty_line() {
        let file = File::open("./res/empty.txt").unwrap();
        let mut lines = Vec::new();

        for line in IntoIterator::into_iter(BufReader::new(file).byte_lines()) {
            let line = line.unwrap();
            let line = String::from_utf8(line).unwrap();

            lines.push(line);
        }

        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], "");
    }

    #[test]
    fn test_buf_read() {
        let buf_reader = BufReader::new(File::open("./res/numbers.txt").unwrap());
        let mut lines = Vec::new();

        for_lend! { line in buf_reader.byte_lines() =>
            let line = line.unwrap();
            let line = String::from_utf8(line.to_vec()).unwrap();
            lines.push(line);
        }

        for i in 0..9 {
            assert_eq!(lines[i], format!("{}", i));
        }
    }
}
