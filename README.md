# bytelines
[![Crates.io](https://img.shields.io/crates/v/bytelines.svg)](https://crates.io/crates/bytelines) [![Build Status](https://img.shields.io/github/workflow/status/whitfin/bytelines/CI)](https://github.com/whitfin/bytelines/actions)

This library provides an easy way to read in input lines as byte slices for high efficiency. It's basically [lines](https://doc.rust-lang.org/std/io/trait.BufRead.html#method.lines) from the standard library, but it reads each line as a byte slice (`&[u8]`). This performs significantly faster than `lines()` in the case you don't particularly care about unicode, and basically as fast as writing the loops out by hand. Although the code itself is somewhat trivial, I've had to roll this in at least 4 tools I've written recently and so I figured it was time to have a convenience crate for it.

### Installation

This tool will be available via [Crates.io](https://crates.io/crates/bytelines), so you can add it as a dependency in your `Cargo.toml`:

```toml
[dependencies]
bytelines = "2.2"
```

### Usage

It's quite simple; in the place you would typically call `lines` on a `BufRead` implementor, you can now call `byte_lines` to retrieve a structure used to walk over lines as `&[u8]` (and thus avoid allocations). There are two ways to use the API, and both are shown below:

```rust
// our input file we're going to walk over lines of, and our reader
let file = File::open("./my-input.txt").expect("able to open file");
let reader = BufReader::new(file);
let mut lines = reader.byte_lines();

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

This interface was introduced in the v2.x lineage of `bytelines`. The `Iterator` trait was previously implemented in v1.x, but required an `unsafe` contract in trying to be too idiomatic. This has since been fixed, and all unsafe code has been removed whilst providing `IntoIterator` implementations for those who prefer the cleaner syntax.
