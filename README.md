# strlinebuf

Simple `#![no_std]` line buffer for separating lines from a byte stream. For example, reading from a serial port byte by byte and assembling into lines.

## Basic Usage

```rust
use strlinebuf::LineBuffer;

// Capacity of the buffer is 10 bytes
let mut line_buffer = LineBuffer::<10>::new();
line_buffer.push_bytes(b"Hello\n").unwrap();

let mut aux_buffer = [0u8; 10];
let bytes_read = line_buffer.read_line_bytes(&mut aux_buffer).unwrap();
let line = core::str::from_utf8(&aux_buffer[..bytes_read]).unwrap();

// line == "Hello"
```

## Configuration

You can also configure the terminator character and the capacity of the buffer.

```rust
use strlinebuf::{LineBuffer, LineBufferConfig, Terminator};

let line_buffer = LineBuffer::<24>::new_with_config(LineBufferConfig {
    terminator: Terminator::CarriageReturn,
});
```
