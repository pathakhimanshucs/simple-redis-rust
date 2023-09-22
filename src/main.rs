use anyhow::Result;
use handler::Value;
use store::HashStore;
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
                println!("error: {}", e);
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
            match command.as_str() {
                "ping" => Value::SimpleString("PONG".to_string()),
                "echo" => args.first().unwrap().clone(),
                "set" => {
                    let mut args_iter = args.into_iter();
                    let key = args_iter.next().unwrap();
                    let value = args_iter.next().unwrap();

                    println!("Set request for key: {:?} and value {:?}", key, value);
                    match (key, value) {
                        (Value::BulkString(key_str), Value::BulkString(value_str)) => {
                            storage.set(key_str, value_str);
                            Value::SimpleString("OK".to_string())
                        }
                        _ => Value::Nil,
                    }
                }
                "get" => {
                    let key = args.into_iter().next().unwrap();
                    if let Value::BulkString(key_str) = key {
                        println!("Get request for key: {:?}", key_str);
                        if let Some(val) = storage.get(key_str) {
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

fn extract_command(value: handler::Value) -> Result<(String, Vec<handler::Value>)> {
    match value {
        handler::Value::Array(a) => Ok((
            unpack_bulk_str(a.first().unwrap().clone())?,
            a.into_iter().skip(1).collect(),
        )),
        _ => Err(anyhow::anyhow!("Unexpected command format")),
    }
}

fn unpack_bulk_str(value: handler::Value) -> Result<String> {
    match value {
        Value::BulkString(s) => Ok(s),
        _ => Err(anyhow::anyhow!("Expected command to be a bulk string")),
    }
}
