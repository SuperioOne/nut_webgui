use tokio::io::{AsyncRead, AsyncWrite};

use crate::{
  CmdName, UpsName, Value, VarName, clients::AsyncNutClient, commands, errors::Error, responses,
};

use super::NutClient;

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
  fn get_attached(
    self,
    ups: &UpsName,
  ) -> impl Future<Output = Result<responses::AttachedDaemons, Error>> {
    self.inner.get_attached(ups)
  }

  fn get_cmd_desc(
    self,
    ups: &UpsName,
    cmd: &CmdName,
  ) -> impl Future<Output = Result<responses::CmdDesc, Error>> {
    self.inner.get_cmd_desc(ups, cmd)
  }

  fn get_protver(self) -> impl Future<Output = Result<responses::ProtVer, Error>> {
    self.inner.get_protver()
  }

  fn get_ups_desc(self, ups: &UpsName) -> impl Future<Output = Result<responses::UpsDesc, Error>> {
    self.inner.get_ups_desc(ups)
  }

  fn get_var(
    self,
    ups: &UpsName,
    var: &VarName,
  ) -> impl Future<Output = Result<responses::UpsVar, Error>> {
    self.inner.get_var(ups, var)
  }

  fn get_var_desc(
    self,
    ups: &UpsName,
    var: &VarName,
  ) -> impl Future<Output = Result<responses::UpsVarDesc, Error>> {
    self.inner.get_var_desc(ups, var)
  }

  fn get_ver(self) -> impl Future<Output = Result<responses::DaemonVer, Error>> {
    self.inner.get_ver()
  }

  fn list_cmd(self, ups: &UpsName) -> impl Future<Output = Result<responses::CmdList, Error>> {
    self.inner.list_cmd(ups)
  }

  fn list_enum(
    self,
    ups: &UpsName,
    var: &VarName,
  ) -> impl Future<Output = Result<responses::EnumList, Error>> {
    self.inner.list_enum(ups, var)
  }

  fn list_ranges(
    self,
    ups: &UpsName,
    var: &VarName,
  ) -> impl Future<Output = Result<responses::RangeList, Error>> {
    self.inner.list_ranges(ups, var)
  }

  fn list_rw(self, ups: &UpsName) -> impl Future<Output = Result<responses::RwList, Error>> {
    self.inner.list_rw(ups)
  }

  fn list_ups(self) -> impl Future<Output = Result<responses::UpsList, Error>> {
    self.inner.list_ups()
  }

  fn list_var(self, ups: &UpsName) -> impl Future<Output = Result<responses::UpsVarList, Error>> {
    self.inner.list_var(ups)
  }

  fn list_client(
    self,
    ups: &UpsName,
  ) -> impl Future<Output = Result<responses::ClientList, Error>> {
    self.inner.list_client(ups)
  }
}

impl<T> NutAuthClient<T>
where
  T: AsyncRead + AsyncWrite + Unpin,
{
  pub async fn fsd(&mut self, ups: &UpsName) -> Result<(), Error> {
    self
      .inner
      .send::<responses::ProtOkFsd>(commands::FsdCommand { ups })
      .await?;

    Ok(())
  }

  pub async fn set_var(
    &mut self,
    ups: &UpsName,
    var: &VarName,
    value: &Value,
  ) -> Result<(), Error> {
    _ = self
      .inner
      .send::<responses::ProtOk>(commands::SetVariable { ups, var, value })
      .await?;

    Ok(())
  }

  pub async fn instcmd(&mut self, ups: &UpsName, cmd: &CmdName) -> Result<(), Error> {
    _ = self
      .inner
      .send::<responses::ProtOk>(commands::InstCmd { ups, cmd })
      .await?;

    Ok(())
  }

  pub fn close(self) -> impl Future<Output = Result<(), Error>> {
    self.inner.close()
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
      .send::<responses::ProtOk>(commands::Username { username })
      .await?;

    _ = client
      .inner
      .send::<responses::ProtOk>(commands::Password { password })
      .await?;

    Ok(client)
  }
}
