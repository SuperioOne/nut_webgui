use std::net::{SocketAddr, ToSocketAddrs};

#[derive(Debug)]
pub enum ServerAddr {
  SocketAddr(SocketAddr),
  Host(String),
}

impl ToSocketAddrs for ServerAddr {
  type Iter = std::vec::IntoIter<SocketAddr>;

  fn to_socket_addrs(&self) -> std::io::Result<Self::Iter> {
    match self {
      ServerAddr::SocketAddr(socket_addr) => Ok(vec![*socket_addr].into_iter()),
      ServerAddr::Host(host) => host.to_socket_addrs(),
    }
  }
}

impl From<String> for ServerAddr {
  #[inline]
  fn from(value: String) -> Self {
    Self::Host(value)
  }
}

impl From<SocketAddr> for ServerAddr {
  #[inline]
  fn from(value: SocketAddr) -> Self {
    Self::SocketAddr(value)
  }
}
