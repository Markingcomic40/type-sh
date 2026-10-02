use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, ToSocketAddrs, UdpSocket};
use std::time::Instant;

use crate::core::config::Rules;
use crate::core::word_pool;
use crate::error::Result;
use crate::net::client::{self, Client};
use crate::net::protocol::ToServer;
use crate::net::room::{Out, Room};
use crate::net::server::Server;

pub const PORT: u16 = 4269;

struct Hosting {
    server: Server,
    room: Room,
}

/// One players end of a room. Same for host and client!
pub struct Session {
    hosting: Option<Hosting>,
    client: Client,
}

impl Session {
    pub fn host(rules: Rules, name: &str) -> Result<Self> {
        let pool = word_pool::load(&rules.wordlist)?;
        let server = Server::bind((Ipv4Addr::UNSPECIFIED, PORT))?;
        let client = Client::connect(SocketAddr::from((Ipv4Addr::LOCALHOST, PORT)))?;

        let session = Self {
            hosting: Some(Hosting {
                server,
                room: Room::new(rules, pool),
            }),
            client,
        };

        session.greet(name);

        Ok(session)
    }

    /// address is an ip or hostname, with the port optional
    pub fn join(address: &str, name: &str) -> Result<Self> {
        let address = address.trim();

        let full = if address.contains(':') {
            address.to_owned()
        } else {
            format!("{address}:{PORT}")
        };

        let addr = full.to_socket_addrs()?.next().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("nothing found at {address}"),
            )
        })?;

        let session = Self {
            hosting: None,
            client: Client::connect(addr)?,
        };

        session.greet(name);

        Ok(session)
    }

    fn greet(&self, name: &str) {
        self.send(&ToServer::Greetings {
            version: env!("CARGO_PKG_VERSION").to_owned(),
            name: name.to_owned(),
        });
    }

    pub fn is_host(&self) -> bool {
        self.hosting.is_some()
    }

    pub fn send(&self, msg: &ToServer) {
        self.client.send(msg);
    }

    /// Runs the room when hosting, then hands back what reached this player
    pub fn poll(&mut self, now: Instant) -> Vec<client::Event> {
        if let Some(Hosting { server, room }) = &mut self.hosting {
            let mut out = Vec::new();

            for event in server.poll() {
                out.extend(room.handle(event, now));
            }

            out.extend(room.tick(now));

            apply(server, out);
        }

        self.client.poll()
    }

    /// Hosting only. words load first so rules naming a list that dont load dont break shi
    pub fn set_rules(&mut self, rules: Rules, now: Instant) -> Result<()> {
        let Some(Hosting { server, room }) = &mut self.hosting else {
            return Ok(());
        };

        let pool = word_pool::load(&rules.wordlist)?;
        apply(server, room.set_rules(rules, pool, now));

        Ok(())
    }
}

fn apply(server: &Server, out: Vec<Out>) {
    for o in out {
        match o {
            Out::To(id, msg) => server.send(id, &msg),
            Out::All(msg) => server.broadcast(&msg),
            Out::Kick(id) => server.kick(id),
        }
    }
}

/// The address others on the network reach this machine at
pub fn lan_ip() -> Option<IpAddr> {
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).ok()?;
    socket.connect(("8.8.8.8", 80)).ok()?;
    socket.local_addr().ok().map(|addr| addr.ip())
}
