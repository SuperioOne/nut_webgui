use super::NutClient;
use crate::{
  CmdName, UpsName, Value, VarName, clients::AsyncNutClient, commands, errors::Error,
  internal::Serialize, responses,
};
use core::borrow::Borrow;
use tokio::{
  io::{AsyncRead, AsyncWrite},
  net::{TcpStream, ToSocketAddrs},
};

pub struct NutAuthClient<T>
where
  T: AsyncRead + AsyncWrite + Unpin,
{
  inner: NutClient<T>,
}

impl<T> AsyncNutClient for &mut NutAuthClient<T>
where
  T: AsyncRead + AsyncWrite + Unpin,
{
  fn get_cmd_desc<N, C>(
    self,
    ups: N,
    cmd: C,
  ) -> impl Future<Output = Result<responses::CmdDesc, Error>>
  where
    N: std::borrow::Borrow<UpsName>,
    C: std::borrow::Borrow<CmdName>,
  {
    self.inner.get_cmd_desc(ups, cmd)
  }

  fn get_protver(self) -> impl Future<Output = Result<responses::ProtVer, Error>> {
    self.inner.get_protver()
  }

  fn get_ups_desc<N>(self, ups: N) -> impl Future<Output = Result<responses::UpsDesc, Error>>
  where
    N: std::borrow::Borrow<UpsName>,
  {
    self.inner.get_ups_desc(ups)
  }

  fn get_var<N, V>(self, ups: N, var: V) -> impl Future<Output = Result<responses::UpsVar, Error>>
  where
    N: std::borrow::Borrow<UpsName>,
    V: std::borrow::Borrow<VarName>,
  {
    self.inner.get_var(ups, var)
  }

  fn get_var_type<N, V>(
    self,
    ups: N,
    var: V,
  ) -> impl Future<Output = Result<responses::UpsVarType, Error>>
  where
    N: std::borrow::Borrow<UpsName>,
    V: std::borrow::Borrow<VarName>,
  {
    self.inner.get_var_type(ups, var)
  }

  fn get_var_desc<N, V>(
    self,
    ups: N,
    var: V,
  ) -> impl Future<Output = Result<responses::UpsVarDesc, Error>>
  where
    N: std::borrow::Borrow<UpsName>,
    V: std::borrow::Borrow<VarName>,
  {
    self.inner.get_var_desc(ups, var)
  }

  fn get_ver(self) -> impl Future<Output = Result<responses::DaemonVer, Error>> {
    self.inner.get_ver()
  }

  fn list_client<N>(self, ups: N) -> impl Future<Output = Result<responses::ClientList, Error>>
  where
    N: std::borrow::Borrow<UpsName>,
  {
    self.inner.list_client(ups)
  }

  fn list_cmd<N>(self, ups: N) -> impl Future<Output = Result<Vec<String>, Error>>
  where
    N: std::borrow::Borrow<UpsName>,
  {
    self.inner.list_cmd(ups)
  }

  fn list_enum<N, V>(
    self,
    ups: N,
    var: V,
  ) -> impl Future<Output = Result<responses::EnumList, Error>>
  where
    N: std::borrow::Borrow<UpsName>,
    V: std::borrow::Borrow<VarName>,
  {
    self.inner.list_enum(ups, var)
  }

  fn list_range<N, V>(
    self,
    ups: N,
    var: V,
  ) -> impl Future<Output = Result<responses::RangeList, Error>>
  where
    N: std::borrow::Borrow<UpsName>,
    V: std::borrow::Borrow<VarName>,
  {
    self.inner.list_range(ups, var)
  }

  fn list_rw<N>(self, ups: N) -> impl Future<Output = Result<responses::RwList, Error>>
  where
    N: std::borrow::Borrow<UpsName>,
  {
    self.inner.list_rw(ups)
  }

  fn list_ups(self) -> impl Future<Output = Result<responses::UpsList, Error>> {
    self.inner.list_ups()
  }

  fn list_var<N>(self, ups: N) -> impl Future<Output = Result<responses::UpsVarList, Error>>
  where
    N: std::borrow::Borrow<UpsName>,
  {
    self.inner.list_var(ups)
  }
}

impl<T> NutAuthClient<T>
where
  T: AsyncRead + AsyncWrite + Unpin,
{
  pub async fn attach<N>(&mut self, ups: N) -> Result<(), Error>
  where
    N: Borrow<UpsName>,
  {
    let command = commands::AttachCommand { ups: ups.borrow() }.serialize();
    self.inner.send::<_, responses::ProtOk>(command).await?;

    Ok(())
  }

  #[inline]
  pub fn detach(self) -> impl Future<Output = Result<(), Error>> {
    self.close()
  }

  pub async fn fsd<N>(&mut self, ups: N) -> Result<(), Error>
  where
    N: Borrow<UpsName>,
  {
    let command = commands::FsdCommand { ups: ups.borrow() }.serialize();
    _ = self.inner.send::<_, responses::ProtOkFsd>(command).await?;

    Ok(())
  }

  pub async fn set_var<N, V, D>(&mut self, ups: N, var: V, value: D) -> Result<(), Error>
  where
    N: Borrow<UpsName>,
    V: Borrow<VarName>,
    D: Borrow<Value>,
  {
    let command = commands::SetVariable {
      ups: ups.borrow(),
      var: var.borrow(),
      value: value.borrow(),
    }
    .serialize();

    _ = self.inner.send::<_, responses::ProtOk>(command).await?;

    Ok(())
  }

  pub async fn instcmd<N, C>(&mut self, ups: N, cmd: C) -> Result<(), Error>
  where
    N: Borrow<UpsName>,
    C: Borrow<CmdName>,
  {
    let command = commands::InstCmd {
      ups: ups.borrow(),
      cmd: cmd.borrow(),
    }
    .serialize();

    _ = self.inner.send::<_, responses::ProtOk>(command).await?;

    Ok(())
  }

  #[inline]
  pub fn is_open(&mut self) -> impl Future<Output = bool> {
    self.inner.is_open()
  }

  pub async fn close(mut self) -> Result<(), Error> {
    _ = self
      .inner
      .send::<_, responses::ProtOkDetach>(commands::DetachCommand.serialize())
      .await?;

    self.inner.close().await?;

    Ok(())
  }
}

impl NutAuthClient<TcpStream> {
  pub async fn connect<A>(addr: A, username: &str, password: &str) -> Result<Self, Error>
  where
    A: ToSocketAddrs,
  {
    let client = NutClient::connect(addr)
      .await?
      .authenticate(username, password)
      .await?;

    Ok(client)
  }
}

impl<T> NutClient<T>
where
  T: AsyncRead + AsyncWrite + Unpin,
{
  pub async fn authenticate(
    self,
    username: &str,
    password: &str,
  ) -> Result<NutAuthClient<T>, Error> {
    let mut client = NutAuthClient { inner: self };
    _ = client
      .inner
      .send::<_, responses::ProtOk>(commands::Username { username }.serialize())
      .await?;

    _ = client
      .inner
      .send::<_, responses::ProtOk>(commands::Password { password }.serialize())
      .await?;

    Ok(client)
  }
}
