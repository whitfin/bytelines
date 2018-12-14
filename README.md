# bytelines
[![Crates.io](https://img.shields.io/crates/v/bytelines.svg)](https://crates.io/crates/bytelines) [![Build Status](https://img.shields.io/travis/whitfin/bytelines.svg?)](https://travis-ci.org/whitfin/bytelines)

This library provides an easy way to read in input lines as byte slices for high efficiency. It's basically [lines](https://doc.rust-lang.org/std/io/trait.BufRead.html#method.lines) from the standard library, but it reads each line as either a byte vector (`Vec<u8>`) or a byte slice (`&[u8]`). Both perform significantly faster than `lines()` in the case you don't particularly care about unicode, while each performs basically as fast as writing the loops out by hand. Although the code itself is somewhat trivial, I've had to roll this in at least 4 tools I've written recently and so I figured it was time to have a convenience crate for it.

### Installation

This tool will be available via [Crates.io](https://crates.io/crates/bytelines), so you can add it as a dependency in your `Cargo.toml`:

```toml
[dependencies]
bytelines = "1.0"
```

### Usage

It's quite simple; in the place you would call `lines` on a `BufRead` implementor, you can now call either `byte_lines` or `ref_byte_lines` to retrieve an iterator of `Vec<u8>` and `&[u8]` respectively.

```rust
let file = File::open("./my-input.txt").expect("able to open file");
for line in BufReader::new(file).byte_lines() {
    // do something with the line
}
```

For places where performance is critical, you should use `ref_byte_lines` over `byte_lines` as it avoids an allocation per input line. This comes at the cost of only being able to safely use this iterator using the `for` style syntax above, and as such the method to retrieve an instance of this iterator is marked `unsafe`. Read the documentation for further descriptions as to why this method is marked `unsafe`.
