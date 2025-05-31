use crate::internal::Serialize;
use crate::{CmdName, UpsName, Value, VarName};

pub struct FsdCommand<'a> {
  pub ups: &'a UpsName,
}

impl Serialize for FsdCommand<'_> {
  type Output = String;

  fn serialize(self) -> Self::Output {
    format!("FSD {}\n", self.ups.as_escaped_str())
  }
}

pub struct GetCmdDesc<'a, 'b> {
  pub ups: &'a UpsName,
  pub cmd: &'b CmdName,
}

impl Serialize for GetCmdDesc<'_, '_> {
  type Output = String;

  fn serialize(self) -> Self::Output {
    format!(
      "GET CMDDESC {ups_name} {cmd_name}\n",
      ups_name = self.ups.as_escaped_str(),
      cmd_name = self.cmd
    )
  }
}

pub struct GetVarDesc<'a, 'b> {
  pub ups: &'a UpsName,
  pub var: &'b VarName,
}

impl Serialize for GetVarDesc<'_, '_> {
  type Output = String;

  fn serialize(self) -> Self::Output {
    format!(
      "GET DESC {ups_name} {var_name}\n",
      ups_name = self.ups.as_escaped_str(),
      var_name = self.var
    )
  }
}

pub struct GetUpsDesc<'a> {
  pub ups: &'a UpsName,
}

impl Serialize for GetUpsDesc<'_> {
  type Output = String;

  fn serialize(self) -> Self::Output {
    format!("GET UPSDESC {}\n", self.ups.as_escaped_str())
  }
}

pub struct GetNumAttach<'a> {
  pub ups: &'a UpsName,
}

impl Serialize for GetNumAttach<'_> {
  type Output = String;

  fn serialize(self) -> Self::Output {
    format!("GET NUMATTACH {}\n", self.ups.as_escaped_str())
  }
}

pub struct GetVar<'a, 'b> {
  pub ups: &'a UpsName,
  pub var: &'b VarName,
}

impl Serialize for GetVar<'_, '_> {
  type Output = String;

  fn serialize(self) -> Self::Output {
    format!(
      "GET VAR {ups_name} {var_name}\n",
      ups_name = self.ups.as_escaped_str(),
      var_name = self.var
    )
  }
}

pub struct GetType<'a, 'b> {
  pub ups: &'a UpsName,
  pub var: &'b VarName,
}

impl Serialize for GetType<'_, '_> {
  type Output = String;

  fn serialize(self) -> Self::Output {
    format!(
      "GET TYPE {ups_name} {var_name}\n",
      ups_name = self.ups.as_escaped_str(),
      var_name = self.var
    )
  }
}

pub struct InstCmd<'a, 'b> {
  pub ups: &'a UpsName,
  pub cmd: &'b CmdName,
}

impl Serialize for InstCmd<'_, '_> {
  type Output = String;

  fn serialize(self) -> Self::Output {
    format!(
      "INSTCMD {ups_name} {cmd_name}\n",
      ups_name = self.ups.as_escaped_str(),
      cmd_name = self.cmd
    )
  }
}

pub struct ListClient<'a> {
  pub ups: &'a UpsName,
}

impl Serialize for ListClient<'_> {
  type Output = String;

  fn serialize(self) -> Self::Output {
    format!("LIST CLIENT {}\n", self.ups.as_escaped_str())
  }
}

pub struct ListCmd<'a> {
  pub ups: &'a UpsName,
}

impl Serialize for ListCmd<'_> {
  type Output = String;

  fn serialize(self) -> Self::Output {
    format!("LIST CMD {}\n", self.ups.as_escaped_str())
  }
}

pub struct ListVar<'a> {
  pub ups: &'a UpsName,
}

impl Serialize for ListVar<'_> {
  type Output = String;

  fn serialize(self) -> Self::Output {
    format!("LIST VAR {}\n", self.ups.as_escaped_str())
  }
}

pub struct ListEnum<'a, 'b> {
  pub ups: &'a UpsName,
  pub var: &'b VarName,
}

impl Serialize for ListEnum<'_, '_> {
  type Output = String;

  fn serialize(self) -> Self::Output {
    format!(
      "LIST ENUM {ups_name} {var_name}\n",
      ups_name = self.ups.as_escaped_str(),
      var_name = self.var
    )
  }
}

pub struct ListRange<'a, 'b> {
  pub ups: &'a UpsName,
  pub var: &'b VarName,
}

impl Serialize for ListRange<'_, '_> {
  type Output = String;

  fn serialize(self) -> Self::Output {
    format!(
      "LIST RANGE {ups_name} {var_name}\n",
      ups_name = self.ups.as_escaped_str(),
      var_name = self.var
    )
  }
}

pub struct ListRw<'a> {
  pub ups: &'a UpsName,
}

impl Serialize for ListRw<'_> {
  type Output = String;

  fn serialize(self) -> Self::Output {
    format!("LIST RW {}\n", self.ups.as_escaped_str())
  }
}

pub struct ListUps;

impl Serialize for ListUps {
  type Output = &'static str;

  fn serialize(self) -> Self::Output {
    "LIST UPS\n"
  }
}

pub struct Password<'a> {
  pub password: &'a str,
}

impl Serialize for Password<'_> {
  type Output = String;

  fn serialize(self) -> Self::Output {
    format!("PASSWORD {}\n", self.password)
  }
}

pub struct Username<'a> {
  pub username: &'a str,
}

impl Serialize for Username<'_> {
  type Output = String;

  fn serialize(self) -> Self::Output {
    format!("USERNAME {}\n", self.username)
  }
}

pub struct SetVariable<'a, 'b, 'c> {
  pub ups: &'a UpsName,
  pub var: &'b VarName,
  pub value: &'c Value,
}

impl Serialize for SetVariable<'_, '_, '_> {
  type Output = String;

  fn serialize(self) -> Self::Output {
    format!(
      "SET VAR {ups_name} {var_name} \"{value}\"\n",
      ups_name = self.ups.as_escaped_str(),
      var_name = self.var,
      value = self.value.as_escaped_str()
    )
  }
}

pub struct GetDaemonVer;

impl Serialize for GetDaemonVer {
  type Output = &'static str;

  fn serialize(self) -> Self::Output {
    "VER\n"
  }
}

pub struct GetProtVer;

impl Serialize for GetProtVer {
  type Output = &'static str;

  fn serialize(self) -> Self::Output {
    "PROTVER\n"
  }
}
