use std::io;
use std::net::{Shutdown, SocketAddr, TcpStream};
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

use super::protocol::{ToClient, ToServer};
use super::{spawn_reader, write_line};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(3);

#[derive(Debug, PartialEq)]
pub enum Event {
    Message(ToClient),
    Closed,
}

pub struct Client {
    stream: TcpStream,
    rx: Receiver<Event>,
}

impl Client {
    pub fn connect(addr: SocketAddr) -> io::Result<Self> {
        let stream = TcpStream::connect_timeout(&addr, CONNECT_TIMEOUT)?;
        stream.set_nodelay(true)?;

        let (tx, rx) = mpsc::channel();

        spawn_reader(stream.try_clone()?, tx, |msg| {
            msg.map_or(Event::Closed, Event::Message)
        });

        Ok(Self { stream, rx })
    }

    /// A failed write shows up as Closed from poll so ignore here
    pub fn send(&self, msg: &ToServer) {
        let _ = write_line(&self.stream, msg);
    }

    pub fn poll(&self) -> Vec<Event> {
        self.rx.try_iter().collect()
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        let _ = self.stream.shutdown(Shutdown::Both);
    }
}
