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
#![doc(html_root_url = "https://docs.rs/bytelines/2.4.0")]
use ::std::io::BufRead;
use ::tokio::io::AsyncBufRead;

// mods
mod std;
mod tokio;
mod util;

// expose all public APIs to keep the v2.x interface the same
pub use crate::std::{ByteLines, ByteLinesIter, ByteLinesReader};
pub use crate::tokio::AsyncByteLines;
pub use lender::*;

/// Creates a new line reader from a stdlib `BufRead`.
#[inline]
pub fn from_std<B>(reader: B) -> ByteLines<B>
where
    B: BufRead,
{
    ByteLines::new(reader)
}

/// Creates a new line reader from a Tokio `AsyncBufRead`.
#[inline]
pub fn from_tokio<B>(reader: B) -> AsyncByteLines<B>
where
    B: AsyncBufRead + Unpin,
{
    AsyncByteLines::new(reader)
}
