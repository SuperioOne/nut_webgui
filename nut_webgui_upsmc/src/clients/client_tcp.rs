use super::NutClient;
use crate::{
  CmdName, UpsName, VarName, commands,
  errors::{Error, ProtocolError},
  internal::{Deserialize, Serialize, lexer::Lexer},
  responses,
};
use core::net::SocketAddr;
use tokio::{
  io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
  net::TcpStream,
};
use tracing::{trace, warn};

pub struct NutTcpClient {
  connection: TcpStream,
  address: SocketAddr,
}

impl NutTcpClient {
  pub async fn connect(addr: SocketAddr) -> Result<Self, Error> {
    let connection = TcpStream::connect(&addr).await?;
    connection.set_nodelay(true)?;

    Ok(Self {
      connection,
      address: addr,
    })
  }

  pub async fn reconnect(&mut self) -> Result<(), Error> {
    if let Err(err) = self.connection.shutdown().await {
      warn!(
        message = "unable to close an existing client connection before reconnecting",
        error = %err
      )
    }

    let connection = TcpStream::connect(&self.address).await?;
    connection.set_nodelay(true)?;

    self.connection = connection;

    Ok(())
  }

  pub async fn close(mut self) -> Result<(), Error> {
    self.connection.shutdown().await?;
    Ok(())
  }

  pub async fn send_raw(&mut self, send: &str) -> Result<String, Error> {
    const LIST_START: &'static str = "BEGIN";
    const PROT_ERR: &'static str = "ERR";

    self.connection.write_all(send.as_bytes()).await?;
    self.connection.flush().await?;

    let mut response_buf = String::new();
    let mut reader = BufReader::new(&mut self.connection);
    let mut start_pos = reader.read_line(&mut response_buf).await?;

    if response_buf.starts_with(LIST_START) {
      let expected_end = format!("END{}", &response_buf[LIST_START.len()..]);

      loop {
        let read = reader.read_line(&mut response_buf).await?;
        let line = &response_buf[start_pos..];

        if line == &expected_end {
          break;
        } else {
          start_pos += read;
        }
      }

      Ok(response_buf)
    } else if response_buf.starts_with(PROT_ERR) {
      let prot_err = ProtocolError::from((&response_buf[PROT_ERR.len()..]).trim());

      Err(prot_err.into())
    } else {
      Ok(response_buf)
    }
  }

  pub(crate) async fn send<T>(
    &mut self,
    command: impl Serialize<Output = impl AsRef<str>>,
  ) -> Result<T, Error>
  where
    T: Deserialize<Error = Error>,
  {
    let cmd_str = command.serialize();
    let response = self.send_raw(cmd_str.as_ref()).await?;

    trace!(
      message = "nut TCP message received",
      response = &response,
      command = cmd_str.as_ref()
    );

    let mut lexer = Lexer::new(&response);

    T::deserialize(&mut lexer)
  }
}

impl NutClient for &mut NutTcpClient {
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

  fn get_cmd_list(self, ups: &UpsName) -> impl Future<Output = Result<responses::CmdList, Error>> {
    let command = commands::ListCmd { ups };
    self.send::<responses::CmdList>(command)
  }

  fn get_enum_list(
    self,
    ups: &UpsName,
    var: &VarName,
  ) -> impl Future<Output = Result<responses::EnumList, Error>> {
    let command = commands::ListEnum { ups, var };
    self.send::<responses::EnumList>(command)
  }

  fn get_range_list(
    self,
    ups: &UpsName,
    var: &VarName,
  ) -> impl Future<Output = Result<responses::RangeList, Error>> {
    let command = commands::ListRange { ups, var };
    self.send::<responses::RangeList>(command)
  }

  fn get_rw_list(self, ups: &UpsName) -> impl Future<Output = Result<responses::RwList, Error>> {
    let command = commands::ListRw { ups };
    self.send::<responses::RwList>(command)
  }

  fn get_ups_desc(self, ups: &UpsName) -> impl Future<Output = Result<responses::UpsDesc, Error>> {
    let command = commands::GetUpsDesc { ups };
    self.send::<responses::UpsDesc>(command)
  }

  fn get_ups_list(self) -> impl Future<Output = Result<responses::UpsList, Error>> {
    self.send::<responses::UpsList>(commands::ListUps)
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

  fn get_var_list(
    self,
    ups: &UpsName,
  ) -> impl Future<Output = Result<responses::UpsVarList, Error>> {
    let command = commands::ListVar { ups };
    self.send::<responses::UpsVarList>(command)
  }

  async fn get_ver(self) -> Result<responses::DaemonVer, Error> {
    let response = self.send_raw(commands::GetDaemonVer.serialize()).await?;
    Ok(responses::DaemonVer {
      ver: response.trim().to_owned(),
    })
  }

  async fn get_protver(self) -> Result<responses::ProtVer, Error> {
    let response = self.send_raw(commands::GetProtVer.serialize()).await?;
    Ok(responses::ProtVer {
      ver: response.trim().to_owned(),
    })
  }
}
