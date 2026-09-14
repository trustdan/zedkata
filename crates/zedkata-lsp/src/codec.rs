use crate::error::LspError;
use std::io::Write;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

const HEADER_SEPARATOR: &[u8] = b"\r\n\r\n";
const CONTENT_LENGTH_PREFIX: &str = "content-length:";

/// Stream reader that parses Language Server Protocol framed messages.
pub struct MessageReader<R> {
    reader: R,
    buffer: Vec<u8>,
}

impl<R: AsyncRead + Unpin> MessageReader<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            buffer: Vec::with_capacity(4096),
        }
    }

    /// Reads the next framed message payload.
    /// Returns `Ok(Some(bytes))` on message, `Ok(None)` on clean EOF, or `Err(LspError)` on error.
    pub async fn read_message(&mut self) -> Result<Option<Vec<u8>>, LspError> {
        let mut temp_buf = [0u8; 2048];

        loop {
            // Check if we have a complete header in our buffer
            if let Some(header_end) = find_subsequence(&self.buffer, HEADER_SEPARATOR) {
                let header_bytes = &self.buffer[..header_end];
                let content_length = parse_content_length(header_bytes)?;

                let body_start = header_end + HEADER_SEPARATOR.len();
                let body_end = body_start + content_length;

                // Check if the entire body has been buffered
                if self.buffer.len() >= body_end {
                    let payload = self.buffer[body_start..body_end].to_vec();
                    // Remove the consumed header and payload from buffer
                    self.buffer.drain(..body_end);
                    return Ok(Some(payload));
                }
            }

            // Need more data from the reader
            let bytes_read = self.reader.read(&mut temp_buf).await?;
            if bytes_read == 0 {
                if self.buffer.is_empty() {
                    return Ok(None); // Clean EOF
                } else {
                    return Err(LspError::Framing(
                        "Unexpected EOF while reading framed LSP message".to_string(),
                    ));
                }
            }
            self.buffer.extend_from_slice(&temp_buf[..bytes_read]);
        }
    }
}

/// Parse Content-Length from raw header bytes.
fn parse_content_length(headers: &[u8]) -> Result<usize, LspError> {
    let header_str = std::str::from_utf8(headers)
        .map_err(|e| LspError::Framing(format!("Invalid UTF-8 in LSP headers: {}", e)))?;

    let mut content_length = None;
    for line in header_str.split("\r\n") {
        let trimmed = line.trim();
        if trimmed
            .to_ascii_lowercase()
            .starts_with(CONTENT_LENGTH_PREFIX)
        {
            let val_str = trimmed[CONTENT_LENGTH_PREFIX.len()..].trim();
            let parsed = val_str.parse::<usize>().map_err(|e| {
                LspError::Framing(format!(
                    "Invalid Content-Length number '{}': {}",
                    val_str, e
                ))
            })?;
            content_length = Some(parsed);
        }
    }

    content_length.ok_or_else(|| {
        LspError::Framing("Missing Content-Length header in LSP message".to_string())
    })
}

/// Asynchronously write an LSP framed message.
pub async fn write_message<W: AsyncWrite + Unpin>(
    writer: &mut W,
    payload: &[u8],
) -> Result<(), LspError> {
    let header = format!("Content-Length: {}\r\n\r\n", payload.len());
    writer.write_all(header.as_bytes()).await?;
    writer.write_all(payload).await?;
    writer.flush().await?;
    Ok(())
}

/// Synchronously write an LSP framed message.
pub fn write_message_sync<W: Write>(writer: &mut W, payload: &[u8]) -> Result<(), LspError> {
    let header = format!("Content-Length: {}\r\n\r\n", payload.len());
    writer.write_all(header.as_bytes())?;
    writer.write_all(payload)?;
    writer.flush()?;
    Ok(())
}

/// Helper to locate a subsequence of bytes.
fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_read_single_framed_message() {
        let msg = b"{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"test\"}";
        let framed = format!(
            "Content-Length: {}\r\n\r\n{}",
            msg.len(),
            std::str::from_utf8(msg).unwrap()
        );

        let mut reader = MessageReader::new(framed.as_bytes());
        let result = reader.read_message().await.unwrap();

        assert_eq!(result, Some(msg.to_vec()));
        assert_eq!(reader.read_message().await.unwrap(), None);
    }

    #[tokio::test]
    async fn test_read_multiple_framed_messages() {
        let msg1 = b"{\"seq\":1}";
        let msg2 = b"{\"seq\":2}";
        let mut framed = Vec::new();
        write_message_sync(&mut framed, msg1).unwrap();
        write_message_sync(&mut framed, msg2).unwrap();

        let mut reader = MessageReader::new(&framed[..]);
        assert_eq!(reader.read_message().await.unwrap(), Some(msg1.to_vec()));
        assert_eq!(reader.read_message().await.unwrap(), Some(msg2.to_vec()));
        assert_eq!(reader.read_message().await.unwrap(), None);
    }

    #[tokio::test]
    async fn test_write_message() {
        let mut output = Vec::new();
        let payload = b"{\"hello\":\"world\"}";
        write_message(&mut output, payload).await.unwrap();

        let expected = format!(
            "Content-Length: {}\r\n\r\n{}",
            payload.len(),
            std::str::from_utf8(payload).unwrap()
        );
        assert_eq!(output, expected.as_bytes());
    }
}
