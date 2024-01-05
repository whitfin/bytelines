# bytelines
[![Build Status](https://img.shields.io/github/actions/workflow/status/whitfin/bytelines/ci.yml)](https://github.com/whitfin/bytelines/actions)
[![Crates.io](https://img.shields.io/crates/v/bytelines.svg)](https://crates.io/crates/bytelines)

This library provides an easy way to read in input lines as byte slices for high efficiency. It's basically [lines](https://doc.rust-lang.org/std/io/trait.BufRead.html#method.lines) from the standard library, but it reads each line as a byte slice (`&[u8]`). This performs significantly faster than `lines()` in the case you don't particularly care about unicode, and basically as fast as writing the loops out by hand. Although the code itself is somewhat trivial, I've had to roll this in at least 4 tools I've written recently and so I figured it was time to have a convenience crate for it.

### Installation

This tool will be available via [Crates.io](https://crates.io/crates/bytelines), so you can add it as a dependency in your `Cargo.toml`:

```toml
[dependencies]
bytelines = "2.5"
```

### Usage

It's quite simple; in the place you would typically call `lines` on a `BufRead` implementor, you can now use `bytelines` to retrieve a structure used to walk over lines as `&[u8]` (and thus avoid allocations). There are two ways to use the API, and both are shown below:

```rust
// our input file we're going to walk over lines of, and our reader
let file = File::open("./my-input.txt").expect("able to open file");
let reader = BufReader::new(file);
let mut lines = ByteLines::new(reader);

// Option 1: Walk using a `while` loop.
//
// This is the most performant option, as it avoids an allocation by
// simply referencing bytes inside the reading structure. This means
// that there's no copying at all, until the developer chooses to.
while let Some(line) = lines.next() {
    // do something with the line
}

// Option 2: Use the `Iterator` trait.
//
// This is more idiomatic, but requires allocating each line into
// an owned `Vec` to avoid potential memory safety issues. Although
// there is an allocation here, the overhead should be negligible
// except in cases where performance is paramount.
for line in lines.into_iter() {
    // do something with the line
}
```

As of v2.3 this crate includes fairly minimal support for Tokio, namely the `AsyncBufRead` trait. This looks fairly similar to the base APIs, and can be used in much the same way.


```rust
// configure our inputs again, using `AsyncByteLines`.
let file = File::open("./my-input.txt").await?;
let reader = BufReader::new(file);
let mut lines = AsyncByteLines::new(reader);

// walk through all lines using a `while` loop
while let Some(line) = lines.next().await? {
    // do something with the line
}

// walk through all lines using `Stream` APIs
lines.into_stream().for_each(|line| {

});
```

The main difference is that the Tokio implementations yield `Result<Option<&[u8]>, _>` instead of `Option<Result<&[u8], _>>` for consistency with the exiting Tokio APIs. If you don't want Tokio support, please disable default features:

```toml
[dependencies]
bytelines = { version = "2.5", default-features = false }
```

This will be removed as a default feature in the next major bump (v3.0), but for now you can exclude it this way.
