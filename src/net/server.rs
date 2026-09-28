use std::collections::HashMap;
use std::io;
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream, ToSocketAddrs};
use std::sync::mpsc::{self, Receiver, Sender};

use super::protocol::{Id, ToClient, ToServer};
use super::{spawn_reader, write_line};

#[derive(Debug, PartialEq)]
pub enum Event {
    Joined(Id),
    Message(Id, ToServer),
    Left(Id),
}

pub struct Server {
    listener: TcpListener,
    peers: HashMap<Id, TcpStream>,
    next_id: Id,
    tx: Sender<Event>,
    rx: Receiver<Event>,
}

impl Server {
    pub fn bind(addr: impl ToSocketAddrs) -> io::Result<Self> {
        let listener = TcpListener::bind(addr)?;

        listener.set_nonblocking(true)?;
        
        let (tx, rx) = mpsc::channel();

        Ok(Self {
            listener,
            peers: HashMap::new(),
            next_id: 0,
            tx,
            rx,
        })
    }

    pub fn addr(&self) -> io::Result<SocketAddr> {
        self.listener.local_addr()
    }

    pub fn poll(&mut self) -> Vec<Event> {
        let mut events = Vec::new();

        // WouldBlock means nobodys waiting; any other error, try next tick
        while let Ok((stream, _)) = self.listener.accept() {
            if let Ok(id) = self.add(stream) {
                events.push(Event::Joined(id));
            }
        }

        for event in self.rx.try_iter() {
            if let Event::Left(id) = event {
                self.peers.remove(&id);
            }

            events.push(event);
        }
        
        events
    }

    /// A failed write shows up as Left from poll so ignore
    pub fn send(&self, id: Id, msg: &ToClient) {
        if let Some(stream) = self.peers.get(&id) {
            let _ = write_line(stream, msg);
        }
    }

    pub fn broadcast(&self, msg: &ToClient) {
        for stream in self.peers.values() {
            let _ = write_line(stream, msg);
        }
    }

    /// Only stops sending, so anything already written still arrives
    pub fn kick(&self, id: Id) {
        if let Some(stream) = self.peers.get(&id) {
            let _ = stream.shutdown(Shutdown::Write);
        }
    }

    fn add(&mut self, stream: TcpStream) -> io::Result<Id> {
        stream.set_nonblocking(false)?;
        stream.set_nodelay(true)?;

        let id = self.next_id;
        
        spawn_reader(stream.try_clone()?, self.tx.clone(), move |msg| match msg {
            Some(msg) => Event::Message(id, msg),
            None => Event::Left(id),
        });
        
        self.peers.insert(id, stream);
        self.next_id += 1;
        
        Ok(id)
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        for stream in self.peers.values() {
            let _ = stream.shutdown(Shutdown::Both);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::thread;
    use std::time::{Duration, Instant};

    use super::*;
    use crate::net::client::{self, Client};

    /// The reader threads need a moment, so keep polling until `count`
    /// events are in or it's clearly not happening
    fn wait<T>(mut poll: impl FnMut() -> Vec<T>, count: usize) -> Vec<T> {
        let deadline = Instant::now() + Duration::from_secs(2);
        let mut events = Vec::new();
        while events.len() < count && Instant::now() < deadline {
            events.extend(poll());
            thread::sleep(Duration::from_millis(5));
        }
        events
    }

    fn setup() -> (Server, Client) {
        let mut server = Server::bind("127.0.0.1:0").unwrap();
        let client = Client::connect(server.addr().unwrap()).unwrap();
        assert_eq!(wait(|| server.poll(), 1), [Event::Joined(0)]);
        (server, client)
    }

    fn greetings(name: &str) -> ToServer {
        ToServer::Greetings {
            version: "0.2.0".into(),
            name: name.into(),
        }
    }

    #[test]
    fn messages_go_both_ways() {
        let (mut server, client) = setup();

        client.send(&greetings("sam"));
        assert_eq!(
            wait(|| server.poll(), 1),
            [Event::Message(0, greetings("sam"))]
        );

        server.broadcast(&ToClient::Welcome { id: 0 });
        assert_eq!(
            wait(|| client.poll(), 1),
            [client::Event::Message(ToClient::Welcome { id: 0 })]
        );
    }

    #[test]
    fn the_server_notices_a_client_leaving() {
        let (mut server, client) = setup();

        drop(client);
        assert_eq!(wait(|| server.poll(), 1), [Event::Left(0)]);
    }

    #[test]
    fn clients_notice_the_server_going_away() {
        let (server, client) = setup();

        drop(server);
        assert_eq!(wait(|| client.poll(), 1), [client::Event::Closed]);
    }

    #[test]
    fn a_kicked_client_still_hears_why() {
        let (server, client) = setup();
        let rejected = ToClient::Rejected {
            reason: "host is on 0.3.0".into(),
        };

        server.send(0, &rejected);
        server.kick(0);
        assert_eq!(
            wait(|| client.poll(), 2),
            [client::Event::Message(rejected), client::Event::Closed]
        );
    }

    #[test]
    fn ids_are_never_reused() {
        let (mut server, first) = setup();
        drop(first);
        assert_eq!(wait(|| server.poll(), 1), [Event::Left(0)]);

        let _second = Client::connect(server.addr().unwrap()).unwrap();
        assert_eq!(wait(|| server.poll(), 1), [Event::Joined(1)]);
    }

    #[test]
    fn a_line_that_isnt_a_message_ends_the_connection() {
        use std::io::Write;

        let mut server = Server::bind("127.0.0.1:0").unwrap();
        let mut raw = TcpStream::connect(server.addr().unwrap()).unwrap();
        raw.write_all(b"GET / HTTP/1.1\n").unwrap();

        assert_eq!(
            wait(|| server.poll(), 2),
            [Event::Joined(0), Event::Left(0)]
        );
    }
}
