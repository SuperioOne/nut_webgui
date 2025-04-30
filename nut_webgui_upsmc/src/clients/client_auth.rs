use crate::{
  CmdName, UpsName, Value, VarName,
  clients::{NutClient, NutTcpClient},
  commands,
  errors::Error,
  responses,
};

pub struct NutAuthClient {
  inner: NutTcpClient,
}

impl NutClient for &mut NutAuthClient {
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

  fn get_cmd_list(
    self,
    ups_name: &UpsName,
  ) -> impl Future<Output = Result<responses::CmdList, Error>> {
    self.inner.get_cmd_list(ups_name)
  }

  fn get_enum_list(
    self,
    ups: &UpsName,
    var: &VarName,
  ) -> impl Future<Output = Result<responses::EnumList, Error>> {
    self.inner.get_enum_list(ups, var)
  }

  fn get_rw_list(self, ups: &UpsName) -> impl Future<Output = Result<responses::RwList, Error>> {
    self.inner.get_rw_list(ups)
  }

  fn get_ups_desc(self, ups: &UpsName) -> impl Future<Output = Result<responses::UpsDesc, Error>> {
    self.inner.get_ups_desc(ups)
  }

  fn get_ups_list(self) -> impl Future<Output = Result<responses::UpsList, Error>> {
    self.inner.get_ups_list()
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

  fn get_var_list(
    self,
    ups_name: &UpsName,
  ) -> impl Future<Output = Result<responses::UpsVarList, Error>> {
    self.inner.get_var_list(ups_name)
  }

  fn get_ver(self) -> impl Future<Output = Result<responses::DaemonVer, Error>> {
    self.inner.get_ver()
  }

  fn get_protver(self) -> impl Future<Output = Result<responses::ProtVer, Error>> {
    self.inner.get_protver()
  }

  fn get_range_list(
    self,
    ups: &UpsName,
    var: &VarName,
  ) -> impl Future<Output = Result<responses::RangeList, Error>> {
    self.inner.get_range_list(ups, var)
  }
}

impl NutAuthClient {
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

  pub async fn reconnect(&mut self, username: &str, password: &str) -> Result<(), Error> {
    _ = self.inner.reconnect().await?;

    _ = self
      .inner
      .send::<responses::ProtOk>(commands::Username { username })
      .await?;

    _ = self
      .inner
      .send::<responses::ProtOk>(commands::Password { password })
      .await?;

    Ok(())
  }

  pub fn close(self) -> impl Future<Output = Result<(), Error>> {
    self.inner.close()
  }
}

impl NutTcpClient {
  pub async fn authenticate(self, username: &str, password: &str) -> Result<NutAuthClient, Error> {
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
