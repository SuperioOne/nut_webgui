use std::net::SocketAddr;
use std::ops::AddAssign;
use tokio::io;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use crate::{extract_error, is_error_response, is_list_end, is_ok_response};
use crate::upsd_client::errors::NutClientErrors;
use crate::upsd_client::{Cmd, Ups, Var};
use crate::upsd_client::parser::{parse_cmd_list, parse_ups_list, parse_var_list};

#[derive(Debug)]
pub struct UpsClient {
  address: SocketAddr,
  connection: TcpStream,
}

#[derive(Debug)]
pub struct UpsAuthClient {
  base_client: UpsClient,
  password: Box<str>,
  username: Box<str>,
}

pub trait Client {
  async fn close(&mut self) -> Result<(), NutClientErrors>;
  async fn reconnect(&mut self) -> Result<(), NutClientErrors>;
  async fn get_ups_list(&mut self) -> Result<Vec<Ups>, NutClientErrors>;
  async fn get_cmd_list(&mut self, ups_name: &str) -> Result<Vec<Cmd>, NutClientErrors>;
  async fn get_var_list(&mut self, ups_name: &str) -> Result<Vec<Var>, NutClientErrors>;
}

impl UpsClient {
  pub async fn create(address: SocketAddr) -> Result<UpsClient, NutClientErrors>
  {
    let connection = TcpStream::connect(address)
      .await
      .map_err(|e| NutClientErrors::IOError(e.kind()))?;

    connection.set_nodelay(true).map_err(|e| NutClientErrors::IOError(e.kind()))?;

    Ok(UpsClient {
      connection,
      address,
    })
  }

  async fn send(&mut self, raw_command: &str) -> Result<BufReader<&mut TcpStream>, NutClientErrors>
  {
    self.connection.writable()
      .await
      .map_err(|err| NutClientErrors::IOError(err.kind()))?;

    self.connection.write_all(raw_command.as_bytes())
      .await
      .map_err(|err| NutClientErrors::IOError(err.kind()))?;

    self.connection.flush()
      .await
      .map_err(|err| NutClientErrors::IOError(err.kind()))?;

    self.connection.readable()
      .await
      .map_err(|err| NutClientErrors::IOError(err.kind()))?;

    Ok(BufReader::new(&mut self.connection))
  }

  pub async fn with_auth(self, username: &str, password: &str) -> Result<UpsAuthClient, NutClientErrors> {
    let auth_client = UpsAuthClient {
      base_client: self,
      username: Box::from(username),
      password: Box::from(password),
    };

    Ok(auth_client)
  }
}

impl Client for UpsClient {
  async fn close(&mut self) -> Result<(), NutClientErrors>
  {
    self.connection.shutdown()
      .await
      .map_err(|e| NutClientErrors::IOError(e.kind()))?;

    Ok(())
  }

  async fn reconnect(&mut self) -> Result<(), NutClientErrors> {
    _ = self.connection.shutdown().await;
    let connection = TcpStream::connect(&self.address)
      .await
      .map_err(|e| NutClientErrors::IOError(e.kind()))?;

    connection.set_nodelay(true).map_err(|e| NutClientErrors::IOError(e.kind()))?;

    self.connection = connection;
    Ok(())
  }

  async fn get_ups_list(&mut self) -> Result<Vec<Ups>, NutClientErrors> {
    let mut reader = self.send("LIST UPS\n").await?;
    let mut message_buffer = String::new();

    loop {
      let mut line_buffer = String::new();

      match reader.read_line(&mut line_buffer).await {
        Ok(0) => {
          break;
        }
        Ok(_) => {
          message_buffer.add_assign(line_buffer.as_str());

          if is_error_response!(&line_buffer) || is_list_end!(&line_buffer) || is_ok_response!(&line_buffer) {
            break;
          }
        }
        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => continue,
        Err(err) => return Err(NutClientErrors::IOError(err.kind()))
      }
    }

    parse_ups_list(&message_buffer)
  }

  async fn get_cmd_list(&mut self, ups_name: &str) -> Result<Vec<Cmd>, NutClientErrors> {
    let command = format!("LIST CMD {0}\n", &ups_name);
    let mut reader = self.send(&command).await?;
    let mut message_buffer = String::new();

    loop {
      let mut line_buffer = String::new();

      match reader.read_line(&mut line_buffer).await {
        Ok(0) => {
          break;
        }
        Ok(_) => {
          message_buffer.add_assign(line_buffer.as_str());

          if is_error_response!(&line_buffer) || is_list_end!(&line_buffer) || is_ok_response!(&line_buffer) {
            break;
          }
        }
        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => continue,
        Err(err) => return Err(NutClientErrors::IOError(err.kind()))
      }
    }

    parse_cmd_list(&message_buffer)
  }

  async fn get_var_list(&mut self, ups_name: &str) -> Result<Vec<Var>, NutClientErrors> {
    let command = format!("LIST VAR {0}\n", &ups_name);
    let mut reader = self.send(&command).await?;
    let mut message_buffer = String::new();

    loop {
      let mut line_buffer = String::new();

      match reader.read_line(&mut line_buffer).await {
        Ok(0) => {
          break;
        }
        Ok(_) => {
          message_buffer.add_assign(line_buffer.as_str());

          if is_error_response!(&line_buffer) || is_list_end!(&line_buffer) || is_ok_response!(&line_buffer) {
            break;
          }
        }
        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => continue,
        Err(err) => return Err(NutClientErrors::IOError(err.kind()))
      }
    }

    parse_var_list(&message_buffer)
  }
}

impl UpsAuthClient {
  pub async fn create(address: SocketAddr, username: &str, password: &str) -> Result<UpsAuthClient, NutClientErrors>
  {
    let connection = TcpStream::connect(address)
      .await
      .map_err(|e| NutClientErrors::IOError(e.kind()))?;

    connection.set_nodelay(true).map_err(|e| NutClientErrors::IOError(e.kind()))?;

    let mut auth_client = UpsAuthClient {
      password: Box::from(password),
      username: Box::from(username),
      base_client: UpsClient {
        connection,
        address,
      },
    };

    _ = auth_client.send_user().await?;
    _ = auth_client.send_pass().await?;

    Ok(auth_client)
  }

  pub async fn send_instcmd(&mut self, ups_name: &str, cmd: &str) -> Result<(), NutClientErrors> {
    let command = format!("INSTCMD {0} {1}\n", ups_name, cmd);
    let mut cmd_result = self.base_client.send(&command).await?;

    Self::is_ok(&mut cmd_result).await
  }

  async fn send_user(&mut self) -> Result<(), NutClientErrors> {
    let user_cmd: &str = &format!("USERNAME {}\n", &self.username);
    let mut user_result = self.base_client.send(user_cmd).await?;

    Self::is_ok(&mut user_result).await
  }

  async fn send_pass(&mut self) -> Result<(), NutClientErrors> {
    let pass_cmd: &str = &format!("PASSWORD {}\n", &self.password);
    let mut password_result = self.base_client.send(pass_cmd).await?;

    Self::is_ok(&mut password_result).await
  }

  async fn is_ok(result: &mut BufReader<&mut TcpStream>) -> Result<(), NutClientErrors> {
    let mut line = String::new();
    _ = result.read_line(&mut line).await;

    if is_error_response!(&line) {
      let error = extract_error!(&line);
      Err(NutClientErrors::ProtocolError(error))
    } else if is_ok_response!(&line) {
      Ok(())
    } else {
      Err(NutClientErrors::EmptyResponse)
    }
  }
}

impl Client for UpsAuthClient {
  async fn close(&mut self) -> Result<(), NutClientErrors> {
    self.base_client.close().await
  }

  async fn reconnect(&mut self) -> Result<(), NutClientErrors> {
    _ = self.base_client.reconnect().await?;
    _ = self.send_user().await?;
    _ = self.send_pass().await?;

    Ok(())
  }

  async fn get_ups_list(&mut self) -> Result<Vec<Ups>, NutClientErrors> {
    self.base_client.get_ups_list().await
  }

  async fn get_cmd_list(&mut self, ups_name: &str) -> Result<Vec<Cmd>, NutClientErrors> {
    self.base_client.get_cmd_list(ups_name).await
  }

  async fn get_var_list(&mut self, ups_name: &str) -> Result<Vec<Var>, NutClientErrors> {
    self.base_client.get_var_list(ups_name).await
  }
}
