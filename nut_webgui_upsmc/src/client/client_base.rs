use crate::{
  CmdName, UpsName, VarName,
  client::AsyncNutClient,
  command,
  error::{Error, ErrorKind, ProtocolError},
  internal::{Deserialize, Serialize, lexer::Lexer},
  response,
};
use core::{borrow::Borrow, time::Duration};
use tokio::{
  io::{
    AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader, Interest, ReadHalf,
    WriteHalf, split,
  },
  net::{TcpStream, ToSocketAddrs},
  time::timeout,
};
use tracing::{error, trace};

pub struct NutClient<S>
where
  S: AsyncRead + AsyncWrite + Unpin + Send + Sync,
{
  reader: BufReader<ReadHalf<S>>,
  writer: WriteHalf<S>,
  timeout: Duration,
}

impl NutClient<TcpStream> {
  pub async fn connect<A>(addr: A) -> Result<Self, Error>
  where
    A: ToSocketAddrs,
  {
    let connection = TcpStream::connect(addr).await?;
    connection.set_nodelay(true)?;
    connection.set_linger(None)?;
    connection
      .ready(Interest::READABLE | Interest::WRITABLE)
      .await?;

    Ok(Self::new(connection))
  }
}

impl<S> From<S> for NutClient<S>
where
  S: AsyncWrite + AsyncRead + Unpin + Send + Sync + 'static,
{
  fn from(value: S) -> Self {
    Self::new(value)
  }
}

impl<S> NutClient<S>
where
  S: AsyncWrite + AsyncRead + Send + Sync + Unpin,
{
  pub fn new(stream: S) -> Self {
    let (reader, writer) = split(stream);
    let reader = BufReader::new(reader);

    Self {
      writer,
      reader,
      timeout: Duration::from_secs(60),
    }
  }

  #[inline]
  pub fn set_timeout(&mut self, timeout: Duration) {
    self.timeout = timeout;
  }

  pub fn into_inner(self) -> S {
    let read_half = self.reader.into_inner();
    let write_half = self.writer;

    // NOTE: no need for `is_pair_of()`
    read_half.unsplit(write_half)
  }

  pub async fn is_open(&mut self) -> bool {
    match self.send_raw(command::GetProtVer.serialize()).await {
      Ok(response) if !response.is_empty() => true,
      _ => false,
    }
  }

  pub async fn close(mut self) -> Result<(), Error> {
    self.writer.shutdown().await?;
    Ok(())
  }

  pub async fn send_raw(&mut self, send: &str) -> Result<String, Error> {
    match timeout(self.timeout, self.inner_send_raw(send)).await {
      Ok(r) => r,
      Err(_) => Err(ErrorKind::RequestTimeout.into()),
    }
  }

  async fn inner_send_raw(&mut self, send: &str) -> Result<String, Error> {
    trace!(message = "tcp message", send = send);
    const LIST_START: &str = "BEGIN LIST";
    const LIST_END: &str = "END LIST";
    const PROT_ERR: &str = "ERR";

    self.writer.write_all(send.as_bytes()).await?;
    self.writer.flush().await?;

    let mut response_buf = String::new();
    let mut start_pos = self.reader.read_line(&mut response_buf).await?;

    if response_buf.starts_with(LIST_START) {
      loop {
        let read = self.reader.read_line(&mut response_buf).await?;
        let line = &response_buf[start_pos..];

        if line.starts_with(LIST_END) {
          break;
        } else {
          start_pos += read;
        }
      }

      trace!(
        message = "upsd tcp protocol: list message received",
        response = &response_buf,
        command = send
      );

      Ok(response_buf)
    } else if let Some(prot_err) = response_buf.strip_prefix(PROT_ERR) {
      let prot_err = ProtocolError::from(prot_err.trim());

      error!(
        message = "upsd tcp protocol: error received",
        response = &response_buf,
        command = send
      );

      Err(prot_err.into())
    } else {
      trace!(
        message = "upsd tcp protocol: line message received",
        response = &response_buf,
        command = send
      );

      Ok(response_buf)
    }
  }

  pub(crate) async fn send<C, R>(&mut self, command: C) -> Result<R, Error>
  where
    R: Deserialize<Error = Error>,
    C: AsRef<str>,
  {
    let response = self.send_raw(command.as_ref()).await?;

    if response.is_empty() {
      Err(ErrorKind::EmptyResponse.into())
    } else {
      let mut lexer = Lexer::new(&response);

      R::deserialize(&mut lexer)
    }
  }
}

impl<S> AsyncNutClient for &mut NutClient<S>
where
  S: AsyncRead + AsyncWrite + Unpin + Send + Sync,
{
  fn get_cmd_desc<N, C>(
    self,
    ups: N,
    cmd: C,
  ) -> impl Future<Output = Result<response::CmdDesc, Error>>
  where
    N: Borrow<UpsName>,
    C: Borrow<CmdName>,
  {
    let command = command::GetCmdDesc {
      ups: ups.borrow(),
      cmd: cmd.borrow(),
    }
    .serialize();

    self.send::<_, response::CmdDesc>(command)
  }

  async fn get_protver(self) -> Result<response::ProtVer, Error> {
    let response = self.send_raw(command::GetProtVer.serialize()).await?;

    if response.is_empty() {
      Err(ErrorKind::EmptyResponse.into())
    } else {
      Ok(response::ProtVer {
        value: response.trim().to_owned(),
      })
    }
  }

  fn get_ups_desc<N>(self, ups: N) -> impl Future<Output = Result<response::UpsDesc, Error>>
  where
    N: Borrow<UpsName>,
  {
    let command = command::GetUpsDesc { ups: ups.borrow() }.serialize();
    self.send::<_, response::UpsDesc>(command)
  }

  fn get_var<N, V>(self, ups: N, var: V) -> impl Future<Output = Result<response::UpsVar, Error>>
  where
    N: Borrow<UpsName>,
    V: Borrow<VarName>,
  {
    let command = command::GetVar {
      ups: ups.borrow(),
      var: var.borrow(),
    }
    .serialize();

    self.send::<_, response::UpsVar>(command)
  }

  fn get_var_type<N, V>(
    self,
    ups: N,
    var: V,
  ) -> impl Future<Output = Result<response::UpsVarType, Error>>
  where
    N: Borrow<UpsName>,
    V: Borrow<VarName>,
  {
    let command = command::GetVarType {
      ups: ups.borrow(),
      var: var.borrow(),
    }
    .serialize();

    self.send::<_, response::UpsVarType>(command)
  }

  fn get_var_desc<N, V>(
    self,
    ups: N,
    var: V,
  ) -> impl Future<Output = Result<response::UpsVarDesc, Error>>
  where
    N: Borrow<UpsName>,
    V: Borrow<VarName>,
  {
    let command = command::GetVarDesc {
      ups: ups.borrow(),
      var: var.borrow(),
    }
    .serialize();

    self.send::<_, response::UpsVarDesc>(command)
  }

  async fn get_ver(self) -> Result<response::DaemonVer, Error> {
    let response = self.send_raw(command::GetDaemonVer.serialize()).await?;

    if response.is_empty() {
      Err(ErrorKind::EmptyResponse.into())
    } else {
      Ok(response::DaemonVer {
        value: response.trim().to_owned(),
      })
    }
  }

  fn list_client<N>(self, ups: N) -> impl Future<Output = Result<response::ClientList, Error>>
  where
    N: Borrow<UpsName>,
  {
    let command = command::ListClient { ups: ups.borrow() }.serialize();
    self.send::<_, response::ClientList>(command)
  }

  fn list_cmd<N>(self, ups: N) -> impl Future<Output = Result<response::CmdList, Error>>
  where
    N: Borrow<UpsName>,
  {
    let command = command::ListCmd { ups: ups.borrow() }.serialize();
    self.send::<_, response::CmdList>(command)
  }

  fn list_enum<N, V>(
    self,
    ups: N,
    var: V,
  ) -> impl Future<Output = Result<response::EnumList, Error>>
  where
    N: Borrow<UpsName>,
    V: Borrow<VarName>,
  {
    let command = command::ListEnum {
      ups: ups.borrow(),
      var: var.borrow(),
    }
    .serialize();

    self.send::<_, response::EnumList>(command)
  }

  fn list_range<N, V>(
    self,
    ups: N,
    var: V,
  ) -> impl Future<Output = Result<response::RangeList, Error>>
  where
    N: Borrow<UpsName>,
    V: Borrow<VarName>,
  {
    let command = command::ListRange {
      ups: ups.borrow(),
      var: var.borrow(),
    }
    .serialize();

    self.send::<_, response::RangeList>(command)
  }

  fn list_rw<N>(self, ups: N) -> impl Future<Output = Result<response::RwList, Error>>
  where
    N: Borrow<UpsName>,
  {
    let command = command::ListRw { ups: ups.borrow() }.serialize();
    self.send::<_, response::RwList>(command)
  }

  fn list_ups(self) -> impl Future<Output = Result<response::UpsList, Error>> {
    self.send::<_, response::UpsList>(command::ListUps.serialize())
  }

  fn list_var<N>(self, ups: N) -> impl Future<Output = Result<response::UpsVarList, Error>>
  where
    N: Borrow<UpsName>,
  {
    let command = command::ListVar { ups: ups.borrow() }.serialize();
    self.send::<_, response::UpsVarList>(command)
  }
}
