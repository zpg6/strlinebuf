#![no_std]

/// Terminator is an enum that represents the different types of terminators that can be used to determine the end of a line.
/// The terminator can be one of the following:
/// - None: No terminator (anything remaining in the buffer is considered part of the line)
/// - CarriageReturn: `\r` (Carriage Return)
/// - Newline: `\n` (Newline)
/// - NULL: `\0` (NULL)
/// - CarriageReturnNewline: `\r\n` (Carriage Return + Newline)
/// - NewlineCarriageReturn: `\n\r` (Newline + Carriage Return)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Terminator {
    /// No terminator, meaning anything remaining in the buffer is considered part of the line.
    None,
    /// `\r` (Carriage Return) as the terminator.
    CarriageReturn,
    /// `\n` (Newline) as the terminator.
    Newline,
    /// `\0` (NULL) as the terminator.
    NULL,
    /// `\r\n` (Carriage Return + Newline) as the terminator.
    /// This is the default terminator for Windows text files.
    CarriageReturnNewline,
    /// `\n\r` (Newline + Carriage Return) as the terminator.
    NewlineCarriageReturn,
}

/// Configuration for the LineBuffer.
pub struct LineBufferConfig {
    /// The terminator character(s) that determines the end of a line.
    pub terminator: Terminator,
}

impl Default for LineBufferConfig {
    fn default() -> Self {
        Self {
            terminator: Terminator::Newline,
        }
    }
}

#[derive(Debug)]
pub enum LineBufferTxError {
    BufferFull,
}

#[derive(Debug)]
pub enum LineBufferRxError {
    BufferEmpty,
    NoLines,
}

/// `LineBuffer` is a simple ring buffer that can be used to store bytes until a line terminator is reached.
pub struct LineBuffer<const CAPACITY: usize> {
    pub buffer: [u8; CAPACITY],
    pub config: LineBufferConfig,
    start: usize,
    end: usize,
    empty: bool,
}

impl<const CAPACITY: usize> LineBuffer<CAPACITY> {
    /// Create a new LineBuffer with the specified capacity and terminator.
    /// Example:
    /// ```rust
    /// use strlinebuf::{LineBuffer, LineBufferConfig, Terminator};
    ///
    /// let line_buffer = LineBuffer::<10>::new();
    /// ```
    /// The above example creates a new LineBuffer with a capacity of 10 and a newline terminator.
    pub fn new() -> Self {
        Self {
            buffer: [0u8; CAPACITY],
            config: LineBufferConfig::default(),
            start: 0,
            end: 0,
            empty: true,
        }
    }

    /// Create a new LineBuffer with the specified capacity and terminator.
    /// Example:
    /// ```rust
    /// use strlinebuf::{LineBuffer, LineBufferConfig, Terminator};
    ///
    /// let line_buffer = LineBuffer::<10>::new_with_config(LineBufferConfig {
    ///    terminator: Terminator::CarriageReturn,
    /// });
    /// ```
    /// The above example creates a new LineBuffer with a capacity of 10 and a CR terminator.
    pub fn new_with_config(config: LineBufferConfig) -> Self {
        Self {
            buffer: [0u8; CAPACITY],
            config,
            start: 0,
            end: 0,
            empty: true,
        }
    }

    /// Check if the buffer is empty.
    /// The buffer is empty if the start and end pointers are equal.
    pub fn is_empty(&self) -> bool {
        self.empty
    }

    /// Check if the buffer is full.
    pub fn is_full(&self) -> bool {
        !self.empty && self.start == self.end
    }

    /// Write a byte to the buffer.
    /// If the buffer is full and allow_overwrites is false, an error will be returned.
    pub fn push_byte(&mut self, byte: u8) -> Result<(), LineBufferTxError> {
        // println!("start: {}, end: {}, byte: {}", self.start, self.end, byte);
        if self.is_full() {
            return Err(LineBufferTxError::BufferFull);
        }

        if self.empty {
            self.empty = false;
        }
        self.buffer[self.end] = byte;
        self.end = (self.end + 1) % CAPACITY;

        Ok(())
    }

    /// Write a slice of bytes to the buffer.
    /// This can be a &[u8] or a &str.ß
    pub fn push_bytes(&mut self, bytes: &[u8]) -> Result<(), LineBufferTxError> {
        for byte in bytes {
            self.push_byte(*byte)?;
        }

        Ok(())
    }

    /// Clear the buffer.
    /// This will reset the start and end pointers to 0 and set the buffer to empty.
    ///
    /// Warning: This will not clear the buffer contents, only the pointers.
    pub fn clear(&mut self) {
        self.start = 0;
        self.end = 0;
        self.empty = true;
    }

    /// Read a line from the buffer into a provided auxiliary buffer of at least the same capacity.
    /// Returns the number of bytes read.
    ///
    /// Note: The transferred contents will not include the terminator character(s).
    pub fn read_line_bytes(
        &mut self,
        aux_buffer: &mut [u8; CAPACITY],
    ) -> Result<usize, LineBufferRxError> {
        if self.is_empty() {
            return Err(LineBufferRxError::BufferEmpty);
        }
        let initial_start = self.start;

        let mut bytes_read = 0;
        loop {
            let next_byte = self.buffer[self.start];
            match self.config.terminator {
                Terminator::None => {}
                Terminator::Newline => {
                    if next_byte == b'\n' {
                        self.start = (self.start + 1) % CAPACITY;
                        if self.start == self.end {
                            self.empty = true;
                        }
                        break;
                    }
                }
                Terminator::CarriageReturn => {
                    if next_byte == b'\r' {
                        self.start = (self.start + 1) % CAPACITY;
                        if self.start == self.end {
                            self.empty = true;
                        }
                        break;
                    }
                }
                Terminator::NULL => {
                    if next_byte == b'\0' {
                        self.start = (self.start + 1) % CAPACITY;
                        if self.start == self.end {
                            self.empty = true;
                        }
                        break;
                    }
                }
                Terminator::CarriageReturnNewline => {
                    if next_byte == b'\r'
                        && self.start != self.end
                        && self.buffer[(self.start + 1) % CAPACITY] == b'\n'
                    {
                        self.start = (self.start + 2) % CAPACITY;
                        if self.start == self.end {
                            self.empty = true;
                        }
                        break;
                    }
                }
                Terminator::NewlineCarriageReturn => {
                    if next_byte == b'\n'
                        && self.start != self.end
                        && self.buffer[(self.start + 1) % CAPACITY] == b'\r'
                    {
                        self.start = (self.start + 2) % CAPACITY;
                        if self.start == self.end {
                            self.empty = true;
                        }
                        break;
                    }
                }
            }

            aux_buffer[bytes_read] = next_byte;
            self.start = (self.start + 1) % CAPACITY;
            bytes_read += 1;

            if self.start == self.end {
                self.empty = true;
                if let Terminator::None = self.config.terminator {
                    break;
                } else {
                    self.start = initial_start;
                    return Err(LineBufferRxError::NoLines);
                }
            }
        }

        Ok(bytes_read)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Simple test to check if the library is working.
    #[test]
    fn test_new() {
        let line_buffer = LineBuffer::<10>::new();

        assert_eq!(line_buffer.buffer, [0; 10]);
        assert_eq!(line_buffer.start, 0);
        assert_eq!(line_buffer.end, 0);
        assert_eq!(line_buffer.config.terminator, Terminator::Newline);
    }

    /// Test pushing a byte at a time, up until and including when the buffer is full.
    #[test]
    fn test_push_byte() {
        let mut line_buffer = LineBuffer::<10>::new();

        line_buffer.push_byte(65).unwrap();

        assert_eq!(line_buffer.buffer, [65, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(line_buffer.start, 0);
        assert_eq!(line_buffer.end, 1);
        assert_eq!(line_buffer.empty, false);

        line_buffer.push_byte(66).unwrap();

        assert_eq!(line_buffer.buffer, [65, 66, 0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(line_buffer.start, 0);
        assert_eq!(line_buffer.end, 2);
        assert_eq!(line_buffer.empty, false);

        line_buffer.push_byte(67).unwrap();
        line_buffer.push_byte(68).unwrap();
        line_buffer.push_byte(69).unwrap();
        line_buffer.push_byte(70).unwrap();
        line_buffer.push_byte(71).unwrap();
        line_buffer.push_byte(72).unwrap();
        line_buffer.push_byte(73).unwrap();
        line_buffer.push_byte(74).unwrap();

        assert_eq!(line_buffer.buffer, [65, 66, 67, 68, 69, 70, 71, 72, 73, 74]);
        assert_eq!(line_buffer.start, 0);
        assert_eq!(line_buffer.end, 0);
        assert_eq!(line_buffer.empty, false);

        line_buffer
            .push_byte(75)
            .expect_err("Expected buffer full error");
    }

    /// Test pushing multiple bytes at a time, up until and including when the buffer is full.
    #[test]
    fn test_push_bytes() {
        // Test adding a full capacity of bytes:
        let mut line_buffer = LineBuffer::<10>::new();

        line_buffer
            .push_bytes(&[65, 66, 67, 68, 69, 70, 71, 72, 73, 74])
            .unwrap();

        assert_eq!(line_buffer.buffer, [65, 66, 67, 68, 69, 70, 71, 72, 73, 74]);
        assert_eq!(line_buffer.start, 0);
        assert_eq!(line_buffer.end, 0);
        assert_eq!(line_buffer.empty, false);
        assert_eq!(line_buffer.is_full(), true);

        line_buffer
            .push_bytes(&[75])
            .expect_err("Expected buffer full error");

        // Test adding more than the capacity of bytes:
        let mut line_buffer = LineBuffer::<10>::new();

        line_buffer
            .push_bytes(&[65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75])
            .expect_err("Expected buffer full error");
    }

    #[test]
    fn test_is_empty() {
        let mut line_buffer = LineBuffer::<10>::new();

        assert_eq!(line_buffer.is_empty(), true);

        line_buffer.push_byte(65).unwrap();

        assert_eq!(line_buffer.is_empty(), false);
    }

    #[test]
    fn test_is_full() {
        let mut line_buffer = LineBuffer::<10>::new();

        assert_eq!(line_buffer.is_full(), false);

        for i in 0..10 {
            line_buffer.push_byte(i as u8).expect("Failed to push byte");
        }

        assert_eq!(line_buffer.is_full(), true);

        line_buffer
            .push_byte(10)
            .expect_err("Expected buffer full error");
    }

    #[test]
    fn test_clear() {
        let mut line_buffer = LineBuffer::<10>::new();

        line_buffer.push_byte(65).unwrap();

        assert_eq!(line_buffer.buffer, [65, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(line_buffer.start, 0);
        assert_eq!(line_buffer.end, 1);
        assert_eq!(line_buffer.empty, false);

        line_buffer.clear();

        // We expect the contents of the buffer to remain the same, but the pointers to be reset.
        assert_eq!(line_buffer.buffer, [65, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(line_buffer.start, 0);
        assert_eq!(line_buffer.end, 0);
        assert_eq!(line_buffer.empty, true);
        assert_eq!(line_buffer.is_empty(), true);
    }

    #[test]
    fn test_read_line() {
        let mut line_buffer = LineBuffer::<10>::new();

        line_buffer.push_bytes(b"Hello\n").unwrap();

        let mut aux_buffer = [0u8; 10];
        let bytes_read = line_buffer.read_line_bytes(&mut aux_buffer).unwrap();

        assert_eq!(bytes_read, 5);
        assert_eq!(aux_buffer, [b'H', b'e', b'l', b'l', b'o', 0, 0, 0, 0, 0]);
        assert_eq!(line_buffer.is_empty(), true);
    }

    #[test]
    fn test_read_line_when_empty() {
        let mut line_buffer = LineBuffer::<10>::new();

        let mut aux_buffer = [0u8; 10];
        line_buffer
            .read_line_bytes(&mut aux_buffer)
            .expect_err("Expected buffer empty error");
    }

    #[test]
    fn test_read_line_with_null_terminator() {
        let mut line_buffer = LineBuffer::<11>::new_with_config(LineBufferConfig {
            terminator: Terminator::NULL,
        });

        line_buffer.push_bytes(b"Hello\0World").unwrap();

        let mut aux_buffer = [0u8; 11];
        let bytes_read = line_buffer.read_line_bytes(&mut aux_buffer).unwrap();

        assert_eq!(bytes_read, 5);
        assert_eq!(aux_buffer, [b'H', b'e', b'l', b'l', b'o', 0, 0, 0, 0, 0, 0]);
        assert_eq!(line_buffer.is_empty(), false);
    }

    #[test]
    fn test_read_line_when_full() {
        let mut line_buffer = LineBuffer::<6>::new();

        line_buffer.push_bytes(b"Hello\n").unwrap();

        let mut aux_buffer = [0u8; 6];
        let bytes_read = line_buffer.read_line_bytes(&mut aux_buffer).unwrap();

        assert_eq!(bytes_read, 5);
        assert_eq!(aux_buffer, [b'H', b'e', b'l', b'l', b'o', 0]);
        assert_eq!(line_buffer.is_empty(), true);

        line_buffer.push_bytes(b"World\n").unwrap();

        let mut aux_buffer = [0u8; 6];
        let bytes_read = line_buffer.read_line_bytes(&mut aux_buffer).unwrap();

        assert_eq!(bytes_read, 5);
        assert_eq!(aux_buffer, [b'W', b'o', b'r', b'l', b'd', 0]);
        assert_eq!(line_buffer.is_empty(), true);
    }

    #[test]
    fn test_read_multiple_lines_after_bulk_insert() {
        let mut line_buffer = LineBuffer::<1024>::new();

        for _i in 0..10 {
            line_buffer.push_bytes(b"Hello World\n").unwrap();
        }
        line_buffer.push_bytes(b"Hello World").unwrap();

        for _i in 0..10 {
            let mut aux_buffer = [0u8; 1024];
            let bytes_read = line_buffer.read_line_bytes(&mut aux_buffer).unwrap();
            assert_eq!(bytes_read, 11);
            assert_eq!(
                aux_buffer[..11],
                [b'H', b'e', b'l', b'l', b'o', b' ', b'W', b'o', b'r', b'l', b'd']
            );
        }

        assert_eq!(line_buffer.is_empty(), false);
    }

    #[test]
    fn test_read_multiple_lines_while_inserting() {
        let mut line_buffer = LineBuffer::<15>::new();

        for _i in 0..100 {
            line_buffer.push_bytes(b"Hello World\n").unwrap();
            let mut aux_buffer = [0u8; 15];
            let bytes_read = line_buffer.read_line_bytes(&mut aux_buffer).unwrap();
            assert_eq!(bytes_read, 11);
            assert_eq!(
                aux_buffer[..11],
                [b'H', b'e', b'l', b'l', b'o', b' ', b'W', b'o', b'r', b'l', b'd']
            );
        }

        assert_eq!(line_buffer.is_empty(), true);
    }

    #[test]
    fn test_read_no_lines() {
        let mut line_buffer = LineBuffer::<10>::new();

        line_buffer.push_bytes(b"Hello").unwrap();

        let mut aux_buffer = [0u8; 10];
        line_buffer
            .read_line_bytes(&mut aux_buffer)
            .expect_err("Expected no lines error");

        line_buffer.push_byte(b'\n').unwrap();

        let mut aux_buffer = [0u8; 10];
        let bytes_read = line_buffer.read_line_bytes(&mut aux_buffer).unwrap();

        assert_eq!(bytes_read, 5);
        assert_eq!(aux_buffer, [b'H', b'e', b'l', b'l', b'o', 0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_read_line_as_str() {
        let mut line_buffer = LineBuffer::<18>::new();

        line_buffer.push_bytes(b"Hello\nHello\nHello\n").unwrap();
        assert_eq!(line_buffer.is_empty(), false);
        assert_eq!(line_buffer.is_full(), true);

        for _ in 0..3 {
            let mut aux_buffer = [0u8; 18];
            let bytes_read = line_buffer.read_line_bytes(&mut aux_buffer).unwrap();
            let line = core::str::from_utf8(&aux_buffer[..bytes_read]).unwrap();
            assert_eq!(bytes_read, 5);
            assert_eq!(line, "Hello");
        }

        assert_eq!(line_buffer.is_empty(), true);
    }

    #[test]
    fn test_read_line_with_carriage_return_newline() {
        let mut line_buffer = LineBuffer::<21>::new_with_config(LineBufferConfig {
            terminator: Terminator::CarriageReturnNewline,
        });

        line_buffer
            .push_bytes(b"Hello\r\nHello\r\nHello\r\n")
            .unwrap();
        assert_eq!(line_buffer.is_empty(), false);
        assert_eq!(line_buffer.is_full(), true);

        for _ in 0..3 {
            let mut aux_buffer = [0u8; 21];
            let bytes_read = line_buffer.read_line_bytes(&mut aux_buffer).unwrap();
            let line = core::str::from_utf8(&aux_buffer[..bytes_read]).unwrap();
            assert_eq!(bytes_read, 5);
            assert_eq!(line, "Hello");
        }

        assert_eq!(line_buffer.is_empty(), true);
    }

    #[test]
    fn test_read_line_with_newline_carriage_return() {
        let mut line_buffer = LineBuffer::<21>::new_with_config(LineBufferConfig {
            terminator: Terminator::NewlineCarriageReturn,
        });

        line_buffer
            .push_bytes(b"Hello\n\rHello\n\rHello\n\r")
            .unwrap();
        assert_eq!(line_buffer.is_empty(), false);
        assert_eq!(line_buffer.is_full(), true);

        for _ in 0..3 {
            let mut aux_buffer = [0u8; 21];
            let bytes_read = line_buffer.read_line_bytes(&mut aux_buffer).unwrap();
            let line = core::str::from_utf8(&aux_buffer[..bytes_read]).unwrap();
            assert_eq!(bytes_read, 5);
            assert_eq!(line, "Hello");
        }

        assert_eq!(line_buffer.is_empty(), true);
    }
}
