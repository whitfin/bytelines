# bytelines
[![Crates.io](https://img.shields.io/crates/v/bytelines.svg)](https://crates.io/crates/bytelines) [![Build Status](https://img.shields.io/travis/whitfin/bytelines.svg?)](https://travis-ci.org/whitfin/bytelines)

This library provides an easy way to read in input lines as byte slices for high efficiency. It's basically [lines](https://doc.rust-lang.org/std/io/trait.BufRead.html#method.lines) from the standard library, but it reads each line as a byte slice (`&[u8]`). This performs significantly faster than `lines()` in the case you don't particularly care about unicode, and basically as fast as writing the loops out by hand. Although the code itself is somewhat trivial, I've had to roll this in at least 4 tools I've written recently and so I figured it was time to have a convenience crate for it.

### Installation

This tool will be available via [Crates.io](https://crates.io/crates/bytelines), so you can add it as a dependency in your `Cargo.toml`:

```toml
[dependencies]
bytelines = "2.0"
```

### Usage

It's quite simple; in the place you would typically call `lines` on a `BufRead` implementor, you can now call `byte_lines` to retrieve a structure used to walk over lines as `&[u8]` (and thus avoid allocations).

```rust
let file = File::open("./my-input.txt").expect("able to open file");
let mut lines = BufReader::new(file).byte_lines();

while let Some(line) in lines.next() {
    // do something with the line
}
```

In the 1.x lineage of `bytelines`, the `Iterator` trait was implemented for `ByteLines` and thus allowed the typical `for $x in $y` syntax. As this required `unsafe` code, this has been removed in favour of the syntax above which allows a completely safe API without impacting the internal performance.
