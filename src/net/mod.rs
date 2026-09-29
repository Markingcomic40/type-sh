pub mod client;
pub mod protocol;
pub mod room;
pub mod server;

use std::io::{self, BufRead, BufReader, Write};
use std::net::TcpStream;
use std::sync::mpsc::Sender;
use std::thread;

use serde::de::DeserializeOwned;
use serde::Serialize;

/// Reads lines until the stream closes/faulty line/no listeners. NONE is last thing we send
fn spawn_reader<T, E>(
    stream: TcpStream,
    tx: Sender<E>,
    wrap: impl Fn(Option<T>) -> E + Send + 'static,
) where
    T: DeserializeOwned,
    E: Send + 'static,
{
    thread::spawn(move || {
        for line in BufReader::new(stream).lines() {
            let Some(msg) = line.ok().and_then(|l| serde_json::from_str(&l).ok()) else {
                break;
            };

            if tx.send(wrap(Some(msg))).is_err() {
                return;
            }
        }

        let _ = tx.send(wrap(None));
    });
}

/// One write per message, so with nodelay each goes out as one packet
fn write_line(mut stream: &TcpStream, msg: &impl Serialize) -> io::Result<()> {
    let mut line = serde_json::to_vec(msg)?;
    line.push(b'\n');
    stream.write_all(&line)
}
