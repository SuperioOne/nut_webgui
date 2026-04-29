use core::{
  ffi::{CStr, FromBytesUntilNulError},
  net::{IpAddr, Ipv4Addr, Ipv6Addr},
  num::NonZeroI32,
  ptr::null_mut,
  str::Utf8Error,
};
use libc::{
  AF_INET, AF_INET6, NI_MAXHOST, NI_NAMEREQD, gai_strerror, getnameinfo, in_addr, in6_addr,
  sa_family_t, sockaddr_in, sockaddr_in6, socklen_t,
};

pub fn lookup_ipv4(ip: Ipv4Addr) -> Result<Box<str>, Error> {
  let addr = sockaddr_in {
    sin_family: AF_INET as sa_family_t,
    sin_port: 0,
    sin_addr: in_addr {
      s_addr: ip.to_bits().to_be(),
    },
    sin_zero: [0; 8],

    #[cfg(target_os = "freebsd")]
    sin_len: size_of::<sockaddr_in>() as u8,
  };

  lookup_inner(&addr)
}

pub fn lookup_ipv6(ip: Ipv6Addr) -> Result<Box<str>, Error> {
  let addr = sockaddr_in6 {
    sin6_family: AF_INET6 as sa_family_t,
    sin6_port: 0,
    sin6_addr: in6_addr {
      s6_addr: ip.to_bits().to_be_bytes(),
    },
    sin6_flowinfo: 0,
    sin6_scope_id: 0,

    #[cfg(target_os = "freebsd")]
    sin6_len: size_of::<sockaddr_in6>() as u8,
  };

  lookup_inner(&addr)
}

#[inline]
pub fn lookup_ip<T>(ip: T) -> Result<Box<str>, Error>
where
  T: Into<IpAddr>,
{
  match ip.into() {
    IpAddr::V4(ipv4_addr) => lookup_ipv4(ipv4_addr),
    IpAddr::V6(ipv6_addr) => lookup_ipv6(ipv6_addr),
  }
}

#[inline]
fn lookup_inner<T>(socket: *const T) -> Result<Box<str>, Error> {
  let mut host = [0u8; NI_MAXHOST as usize];
  let errcode = unsafe {
    getnameinfo(
      socket.cast(),
      size_of::<T>() as socklen_t,
      host.as_mut_ptr().cast(),
      NI_MAXHOST,
      null_mut(),
      0,
      NI_NAMEREQD,
    )
  };

  match NonZeroI32::new(errcode) {
    Some(v) => Err(Error::from(v)),
    None => {
      let cstr = CStr::from_bytes_until_nul(&host)?;
      Ok(Box::from(cstr.to_str()?))
    }
  }
}

#[derive(Debug)]
pub enum Error {
  InvalidCStr,
  InvalidUtf8,
  UnspecifiedError,
  NameInfoError { inner: &'static CStr },
}

impl core::fmt::Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Error::InvalidCStr => f.write_str("invalid cstr returned"),
      Error::InvalidUtf8 => f.write_str("cstr not a valid utf-8"),
      Error::UnspecifiedError => f.write_str("lookup failed with no error message"),
      Error::NameInfoError { inner } => f.write_str(inner.to_string_lossy().as_ref()),
    }
  }
}

impl core::error::Error for Error {}

impl From<FromBytesUntilNulError> for Error {
  #[inline]
  fn from(_: FromBytesUntilNulError) -> Self {
    Self::InvalidCStr
  }
}

impl From<Utf8Error> for Error {
  #[inline]
  fn from(_: Utf8Error) -> Self {
    Self::InvalidUtf8
  }
}

impl From<NonZeroI32> for Error {
  #[inline]
  fn from(errcode: NonZeroI32) -> Self {
    let strerr_ptr = unsafe { gai_strerror(errcode.get()) };

    // NOTE: gai_strerror returns C string with lifetime of 'static for MUSL, Glibc, and FreeBSD.
    if !strerr_ptr.is_null() {
      Self::NameInfoError {
        inner: unsafe { CStr::from_ptr(strerr_ptr) },
      }
    } else {
      Self::UnspecifiedError
    }
  }
}

#[cfg(test)]
mod test {
  use crate::sync::reverse_dns::{Error, lookup_ip};
  use std::{
    net::{Ipv4Addr, Ipv6Addr},
    str::FromStr,
  };

  #[test]
  #[ignore = "must be explicitly enabled"]
  fn lookup_cloudflare_dns_ipv4() {
    match lookup_ip(Ipv4Addr::new(1, 1, 1, 1)) {
      Ok(result) => assert_eq!("one.one.one.one", result.as_ref()),
      Err(err) => assert!(false, "cannot resolve cloudflare IPv4 DNS, {}", err),
    }
  }

  #[test]
  #[ignore = "must be explicitly enabled"]
  fn lookup_failure() {
    match lookup_ip(Ipv4Addr::new(255, 255, 255, 255)) {
      Ok(_) => assert!(false, "lookup should've failed"),
      Err(Error::NameInfoError { .. }) => {
        assert!(true)
      }
      Err(err) => assert!(false, "lookup failed but error type is not correct {}", err),
    }
  }

  #[test]
  #[ignore = "must be explicitly enabled"]
  fn lookup_google_dns_ipv4() {
    match lookup_ip(Ipv4Addr::new(8, 8, 8, 8)) {
      Ok(result) => assert_eq!("dns.google", result.as_ref()),
      Err(err) => assert!(false, "cannot resolve google DNS, {}", err),
    }
  }

  #[test]
  #[ignore = "must be explicitly enabled"]
  fn lookup_cloudflare_dns_ipv6() {
    match lookup_ip(Ipv6Addr::from_str("2606:4700:4700::1111").unwrap()) {
      Ok(result) => assert_eq!("one.one.one.one", result.as_ref()),
      Err(err) => assert!(false, "cannot resolve cloudflare IPv6 DNS, {}", err),
    }
  }
}
