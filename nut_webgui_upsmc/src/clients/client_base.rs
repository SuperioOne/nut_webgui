use super::AsyncNutClient;
use crate::{
  CmdName, UpsName, VarName, commands,
  errors::{Error, ErrorKind, ProtocolError},
  internal::{Deserialize, Serialize, lexer::Lexer},
  responses,
};
use core::net::SocketAddr;
use tokio::{
  io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader},
  net::TcpStream,
};
use tracing::trace;

pub struct NutClient<S>
where
  S: AsyncRead + AsyncWrite,
{
  stream: S,
}

impl NutClient<TcpStream> {
  pub async fn connect(addr: SocketAddr) -> Result<Self, Error> {
    let connection = TcpStream::connect(&addr).await?;
    connection.set_nodelay(true)?;

    Ok(Self { stream: connection })
  }
}

impl<S> NutClient<S>
where
  S: AsyncWrite + AsyncRead + Unpin,
{
  pub fn new(stream: S) -> Self {
    Self { stream }
  }

  pub async fn is_open(&mut self) -> bool {
    match self.send_raw(commands::GetProtVer.serialize()).await {
      Ok(response) if !response.is_empty() => true,
      _ => false,
    }
  }

  pub async fn close(mut self) -> Result<(), Error> {
    self.stream.shutdown().await?;
    Ok(())
  }

  pub async fn send_raw(&mut self, send: &str) -> Result<String, Error> {
    trace!(message = "tcp message", send = send);
    const LIST_START: &'static str = "BEGIN LIST";
    const LIST_END: &'static str = "END LIST";
    const PROT_ERR: &'static str = "ERR";

    self.stream.write_all(send.as_bytes()).await?;
    self.stream.flush().await?;

    let mut response_buf = String::new();
    let mut reader = BufReader::new(&mut self.stream);
    let mut start_pos = reader.read_line(&mut response_buf).await?;

    if response_buf.starts_with(LIST_START) {
      loop {
        let read = reader.read_line(&mut response_buf).await?;
        let line = &response_buf[start_pos..];

        if line.starts_with(LIST_END) {
          break;
        } else {
          start_pos += read;
        }
      }

      trace!(
        message = "nut tcp list message received",
        response = &response_buf,
        command = send
      );

      Ok(response_buf)
    } else if response_buf.starts_with(PROT_ERR) {
      let prot_err = ProtocolError::from((&response_buf[PROT_ERR.len()..]).trim());

      trace!(
        message = "upsd tcp prot error received",
        response = &response_buf,
        command = send
      );

      Err(prot_err.into())
    } else {
      trace!(
        message = "nut tcp line message received",
        response = &response_buf,
        command = send
      );

      Ok(response_buf)
    }
  }

  pub(crate) async fn send<R>(
    &mut self,
    command: impl Serialize<Output = impl AsRef<str>>,
  ) -> Result<R, Error>
  where
    R: Deserialize<Error = Error>,
  {
    let cmd_str = command.serialize();
    let response = self.send_raw(cmd_str.as_ref()).await?;

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
  S: AsyncRead + AsyncWrite + Unpin,
{
  fn get_attached(
    self,
    ups: &UpsName,
  ) -> impl Future<Output = Result<responses::AttachedDaemons, Error>> {
    let command = commands::GetNumAttach { ups };
    self.send::<responses::AttachedDaemons>(command)
  }

  fn get_cmd_desc(
    self,
    ups: &UpsName,
    cmd: &CmdName,
  ) -> impl Future<Output = Result<responses::CmdDesc, Error>> {
    let command = commands::GetCmdDesc { ups, cmd };
    self.send::<responses::CmdDesc>(command)
  }

  async fn get_protver(self) -> Result<responses::ProtVer, Error> {
    let response = self.send_raw(commands::GetProtVer.serialize()).await?;

    if response.is_empty() {
      Err(ErrorKind::EmptyResponse.into())
    } else {
      Ok(responses::ProtVer {
        value: response.trim().to_owned(),
      })
    }
  }

  fn get_ups_desc(self, ups: &UpsName) -> impl Future<Output = Result<responses::UpsDesc, Error>> {
    let command = commands::GetUpsDesc { ups };
    self.send::<responses::UpsDesc>(command)
  }

  fn get_var(
    self,
    ups: &UpsName,
    var: &VarName,
  ) -> impl Future<Output = Result<responses::UpsVar, Error>> {
    let command = commands::GetVar { ups, var };
    self.send::<responses::UpsVar>(command)
  }

  fn get_var_desc(
    self,
    ups: &UpsName,
    var: &VarName,
  ) -> impl Future<Output = Result<responses::UpsVarDesc, Error>> {
    let command = commands::GetVarDesc { ups, var };
    self.send::<responses::UpsVarDesc>(command)
  }

  async fn get_ver(self) -> Result<responses::DaemonVer, Error> {
    let response = self.send_raw(commands::GetDaemonVer.serialize()).await?;

    if response.is_empty() {
      Err(ErrorKind::EmptyResponse.into())
    } else {
      Ok(responses::DaemonVer {
        value: response.trim().to_owned(),
      })
    }
  }

  fn list_client(
    self,
    ups: &UpsName,
  ) -> impl Future<Output = Result<responses::ClientList, Error>> {
    let command = commands::ListClient { ups };
    self.send::<responses::ClientList>(command)
  }

  fn list_cmd(self, ups: &UpsName) -> impl Future<Output = Result<responses::CmdList, Error>> {
    let command = commands::ListCmd { ups };
    self.send::<responses::CmdList>(command)
  }

  fn list_enum(
    self,
    ups: &UpsName,
    var: &VarName,
  ) -> impl Future<Output = Result<responses::EnumList, Error>> {
    let command = commands::ListEnum { ups, var };
    self.send::<responses::EnumList>(command)
  }

  fn list_range(
    self,
    ups: &UpsName,
    var: &VarName,
  ) -> impl Future<Output = Result<responses::RangeList, Error>> {
    let command = commands::ListRange { ups, var };
    self.send::<responses::RangeList>(command)
  }

  fn list_rw(self, ups: &UpsName) -> impl Future<Output = Result<responses::RwList, Error>> {
    let command = commands::ListRw { ups };
    self.send::<responses::RwList>(command)
  }

  fn list_ups(self) -> impl Future<Output = Result<responses::UpsList, Error>> {
    self.send::<responses::UpsList>(commands::ListUps)
  }

  fn list_var(self, ups: &UpsName) -> impl Future<Output = Result<responses::UpsVarList, Error>> {
    let command = commands::ListVar { ups };
    self.send::<responses::UpsVarList>(command)
  }
}

impl<S> From<S> for NutClient<S>
where
  S: AsyncWrite + AsyncRead + Unpin,
{
  fn from(value: S) -> Self {
    Self { stream: value }
  }
}
