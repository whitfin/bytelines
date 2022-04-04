//! Module exposing utility handlers across read types.
use std::io::Result;

/// Handles a line of input and maps into the provided buffer and returns a reference.
pub fn handle_line(input: Result<usize>, buffer: &mut Vec<u8>) -> Option<Result<&[u8]>> {
    match input {
        // short circuit on error
        Err(e) => Some(Err(e)),

        // no input, done
        Ok(0) => None,

        // bytes!
        Ok(mut n) => {
            // always "pop" the delim
            if buffer[n - 1] == b'\n' {
                n -= 1;
                // also "pop" a potential leading \r
                if n > 0 && buffer[n - 1] == b'\r' {
                    n -= 1;
                }
            }

            // pass back the byte slice
            Some(Ok(&buffer[..n]))
        }
    }
}
