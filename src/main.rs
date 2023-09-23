use anyhow::Result;
use handler::Value;
use store::{get_epoch_ms, HashStore, RedisValue};
use tokio::net::{TcpListener, TcpStream};

mod handler;
mod store;

#[tokio::main]
async fn main() {
    let storage = HashStore::new();
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    loop {
        let stream = listener.accept().await;
        let store_clone = storage.clone();
        match stream {
            Ok((stream, _)) => {
                tokio::spawn(async move { handle_stream(stream, store_clone).await });
            }
            Err(e) => {
                println!("error: {e}");
            }
        }
    }
}

async fn handle_stream(stream: TcpStream, storage: impl store::RedisKV) {
    let mut handler = handler::ResponseHandler::new(stream);

    loop {
        let value = handler.read_value().await.unwrap();

        let response = if let Some(v) = value {
            let (command, args) = extract_command(v).unwrap();
            // TODO: Refactor command handling
            match command.as_str() {
                "ping" => Value::SimpleString("PONG".to_string()),
                "echo" => args.first().unwrap().clone(),
                "set" => match args.as_slice() {
                    [Value::BulkString(key_str), Value::BulkString(value_str), Value::BulkString(opt), Value::BulkString(expiry)]
                        if opt == "px" =>
                    {
                        let expiry_ms = expiry.parse::<u64>().unwrap();

                        let val = RedisValue::ValueWithExpiry {
                            value: value_str.to_owned(),
                            expiry_unix_ms: get_epoch_ms() + expiry_ms as u128,
                        };
                        storage.set(key_str.to_owned(), val);
                        Value::SimpleString("OK".to_string())
                    }
                    [Value::BulkString(key_str), Value::BulkString(value_str), ..] => {
                        let val = RedisValue::SimpleValue(value_str.to_owned());
                        storage.set(key_str.to_owned(), val);
                        Value::SimpleString("OK".to_string())
                    }
                    _ => Value::Nil,
                },
                "get" => {
                    if let Some(Value::BulkString(key_str)) = args.first() {
                        if let Some(val) = storage.get(key_str.clone()) {
                            Value::SimpleString(val)
                        } else {
                            Value::Nil
                        }
                    } else {
                        Value::Nil
                    }
                }
                c => panic!("cannot handle command {}", c),
            }
        } else {
            break;
        };

        handler.write_value(response).await.unwrap();
    }
}

fn extract_command(value: handler::Value) -> Result<(String, Vec<Value>)> {
    match value {
        handler::Value::Array(a) => {
            let mut iter = a.into_iter();
            Ok((
                iter.next()
                    .and_then(unpack_bulk_str)
                    .ok_or(anyhow::anyhow!("Expected command to be a bulk string"))?,
                iter.collect(),
            ))
        }
        _ => Err(anyhow::anyhow!("Unexpected command format")),
    }
}

fn unpack_bulk_str(value: handler::Value) -> Option<String> {
    match value {
        Value::BulkString(s) => Some(s),
        _ => None,
    }
}
