use anyhow::Result;
use bytes::BytesMut;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
pub struct ResponseHandler {
    stream: TcpStream,
    buffer: BytesMut,
}

impl ResponseHandler {
    pub fn new(stream: TcpStream) -> Self {
        ResponseHandler {
            stream,
            buffer: BytesMut::with_capacity(512),
        }
    }

    pub async fn read_value(&mut self) -> Result<Option<Value>> {
        let bytes_read = self.stream.read_buf(&mut self.buffer).await?;
        if bytes_read == 0 {
            return Ok(None);
        }

        let (msg, _) = parse_message(self.buffer.split())?;
        Ok(Some(msg))
    }

    pub async fn write_value(&mut self, value: Value) -> Result<()> {
        _ = self.stream.write(value.serialize()?.as_bytes()).await?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub enum Value {
    SimpleString(String),
    BulkString(String),
    Array(Vec<Value>),
    Nil,
}

impl Value {
    pub fn serialize(self) -> Result<String> {
        match self {
            Value::SimpleString(s) => Ok(format!("+{}\r\n", s)),
            Value::BulkString(s) => Ok(format!("${}\r\n{}\r\n", s.len(), s)),
            Value::Nil => Ok("$-1\r\n".to_string()),
            _ => Err(anyhow::anyhow!("Unsupported value for serialize")),
        }
    }
}

fn parse_message(buffer: BytesMut) -> Result<(Value, usize)> {
    match buffer[0] as char {
        '+' => parse_simple_string(buffer),
        '*' => parse_array(buffer),
        '$' => parse_bulk_string(buffer),
        _ => Err(anyhow::anyhow!("Not a known value type {buffer:?}")),
    }
}

fn read_until_crlf(buffer: &[u8]) -> Option<&[u8]> {
    buffer
        .windows(2)
        .position(|w| w[0] == b'\r' && w[1] == b'\n')
        .map(|i| &buffer[0..i])
}

fn parse_simple_string(buffer: BytesMut) -> Result<(Value, usize)> {
    let line = read_until_crlf(&buffer[1..]).ok_or(anyhow::anyhow!("Invalid string {buffer:?}"))?;
    let string = String::from_utf8(line.to_vec())?;
    Ok((Value::SimpleString(string), line.len() + 1))
}

fn parse_array(buffer: BytesMut) -> Result<(Value, usize)> {
    let line =
        read_until_crlf(&buffer[1..]).ok_or(anyhow::anyhow!("Invalid array format {buffer:?}"))?;
    let array_length = parse_int(line)?;

    let mut bytes_consumed = line.len() + 1;
    let mut items = vec![];
    for _ in 0..array_length {
        let (array_item, length) = parse_message(BytesMut::from(&buffer[bytes_consumed..]))?;
        bytes_consumed += length;
        items.push(array_item);
    }

    Ok((Value::Array(items), bytes_consumed))
}

fn parse_bulk_string(buffer: BytesMut) -> Result<(Value, usize)> {
    let line =
        read_until_crlf(&buffer[1..]).ok_or(anyhow::anyhow!("Invalid array format {buffer:?}"))?;
    let bulk_string_length = parse_int(line)?;

    let bytes_consumed = line.len() + 1;
    let end_of_bulk = bytes_consumed + bulk_string_length as usize;
    let total_parsed = end_of_bulk + 2;

    let string = String::from_utf8(buffer[bytes_consumed..end_of_bulk].to_vec())?;
    Ok((Value::BulkString(string), total_parsed))
}

fn parse_int(buffer: &[u8]) -> Result<i64> {
    Ok(String::from_utf8(buffer.to_vec())?.parse::<i64>()?)
}
