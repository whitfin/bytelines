//! `Bytelines` is a simple library crate which offers line iteration for
//! `BufRead` via `&[u8]` rather than `String`.
//!
//! Due to the removal of checking for `String` validity, this is typically
//! much faster for reading in raw data and much more flexible. The APIs
//! offered in this crate are intended to function exactly the same as the
//! `lines` function inside the `BufRead` trait, except that the bytes which
//! precede the line delimiter are not validated.
//!
//! Performance of [ByteLines](enum.ByteLines.html) is very close to that of
//! writing a `loop` manually, whereas [RefByteLines](enum.RefByteLines.html)
//! is practically identical due to the avoidance of "unnecessary" allocations.
use std::io::BufRead;
use std::marker::PhantomData;

/// Represents anything which can provide iterators of byte lines.
pub trait ByteLinesReader<'a, B>
where
    B: BufRead,
{
    /// Returns an iterator over the lines of this reader (as `Vec<u8>`).
    ///
    /// Just like the equivalent in the standard library, the iterator returned
    /// from this function will yield instances of `io::Result<String>`. Each
    /// string returned will not have a newline byte (the 0xA byte) or CRLF
    /// (0xD, 0xA bytes) at the end.
    fn byte_lines(self) -> ByteLines<'a, B>;

    /// Returns an iterator over the lines of this reader (as `&[u8]`).
    ///
    /// This method operates in the same way as [byte_lines](#method.byte_lines),
    /// except that the iterated values are references to the internal byte buffer.
    /// Due to this, you can only safely hold a single line at any given time, and
    /// as such this method is marked as `unsafe`. If you're using usual loop syntax
    /// of `for $x in $y` your code will not come across this unsafe contract.
    ///
    /// When performance is important, this method should be used rather than
    /// [byte_lines](#method.byte_lines) as there is only a single buffer
    /// allocation (disregarding any potential resizing that may be required),
    /// whereas [byte_lines](#method.byte_lines) will allocate a `Vec<u8>` for
    /// each input line and provide ownership.
    unsafe fn ref_byte_lines(self) -> RefByteLines<'a, B>;
}

/// Blanket implementation for all `BufRead`.
impl<'a, B> ByteLinesReader<'a, B> for B
where
    B: BufRead,
{
    /// Returns an iterator over the lines of this reader (as `Vec<u8>`).
    fn byte_lines(self) -> ByteLines<'a, Self> {
        ByteLines {
            inner: unsafe { self.ref_byte_lines() },
        }
    }

    /// Returns an iterator over the lines of this reader (as `&[u8]`).
    unsafe fn ref_byte_lines(self) -> RefByteLines<'a, Self> {
        RefByteLines {
            buffer: Vec::new(),
            reader: self,
            marker: PhantomData,
        }
    }
}

/// Provides a safe iterator over lines of input as byte vectors (`Vec<u8>`).
///
/// Internally, this iterator delegates to `RefByteLines` - the only difference
/// being that this iterator will allocate a vector for each reference returned,
/// thus making ownership clear and avoiding any issues with data races.
pub struct ByteLines<'a, B>
where
    B: BufRead,
{
    inner: RefByteLines<'a, B>,
}

/// Provides an iterator over lines of input as byte slices (`&[u8]`).
///
/// This iterator requires opting in to the use of unsafe code, as there is a
/// potential data race if you call `next()` on the iterator twice. This iterator
/// should only be used in a traditional `for $x in $y` syntax, otherwise values
/// cannot be relied upon as being consistent.
///
/// Here is a demonstration of this issue in action using a very basic clash of
/// the same length. Note that you might (in some cases) get mixed input if you
/// went from a longer length value to a shorter length.
///
/// ```rust
/// unsafe {
///     # construct our iterator from our file input (1\n2\n3)
///     let mut iter = BufReader::new(file).ref_byte_lines();
///
///     # take the first line from the input
///     let line1 = iter.next();
///     println!("{:?}", line1); // equivalent to bytes of "1"
///
///     # take the second line from the input
///     let line2 = iter.next();
///     println!("{:?}", line2); // equivalent to bytes of "2"
///     println!("{:?}", line1); // also now equivalent to bytes of "2"
/// }
/// ```
///
/// This implmentation is much more memory efficient than `ByteLines` (and more
/// performant), and so should be used in performance critical code blocks. As
/// a small aside, `ByteLines` simply delegates to this struct internally and
/// provides an allocation on top to enforce all ownership correctly.
pub struct RefByteLines<'a, B>
where
    B: BufRead,
{
    buffer: Vec<u8>,
    marker: PhantomData<&'a B>,
    reader: B,
}

/// Wrapping iterator to enforce ownership.
impl<'a, B> Iterator for ByteLines<'a, B>
where
    B: BufRead,
{
    type Item = Result<Vec<u8>, std::io::Error>;

    /// Retrieves the next line in the iterator (if any).
    fn next(&mut self) -> Option<Result<Vec<u8>, std::io::Error>> {
        self.inner.next().map(|r| r.map(|s| s.to_vec()))
    }
}

/// Base iterator for line retrieval.
impl<'a, B> Iterator for RefByteLines<'a, B>
where
    B: BufRead,
{
    type Item = Result<&'a [u8], std::io::Error>;

    /// Retrieves the next line in the iterator (if any).
    fn next(&mut self) -> Option<Result<&'a [u8], std::io::Error>> {
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

                // Here's the fun unsafe section; in order to provide a reference and avoid allocation,
                // we need to extend the lifetime and so we do so here. This means that you're open to
                // data races in the case you call `next` on an iterator twice, and maintain the values
                // of each retrieved line (as the former will be invalidated to point to the bytes of
                // the second). To avoid this, simply always use `for $x in $y` syntax when using this
                // type of iteration directly (as you're never going to hold two lines at once).
                unsafe {
                    Some(Ok(std::mem::transmute::<&[u8], &'a [u8]>(
                        &self.buffer[..n],
                    )))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::BufReader;

    #[test]
    fn test_basic_iterator() {
        let file = File::open("./res/numbers.txt").unwrap();

        let lines: Vec<String> = BufReader::new(file)
            .byte_lines()
            .map(|line| line.unwrap())
            .map(|line| String::from_utf8(line).unwrap())
            .collect();

        for i in 0..9 {
            assert_eq!(lines[i], format!("{}", i));
        }
    }
}
