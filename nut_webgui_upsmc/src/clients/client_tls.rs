use crate::{
  clients::NutClient, commands::StartTls, errors::Error, internal::Serialize, responses::ProtOkTls,
};
use std::sync::Arc;
use tokio::net::{TcpStream, ToSocketAddrs};
use tokio_rustls::{
  TlsConnector,
  client::TlsStream,
  rustls::{ClientConfig, pki_types::ServerName},
};

impl NutClient<TlsStream<TcpStream>> {
  pub async fn connect_with_tls<A>(
    addr: A,
    srv_name: ServerName<'static>,
    config: Arc<ClientConfig>,
  ) -> Result<Self, Error>
  where
    A: ToSocketAddrs,
  {
    let connector = TlsConnector::from(config);

    let mut client = NutClient::connect(addr).await?;
    client.send::<_, ProtOkTls>(StartTls.serialize()).await?;

    let connection = connector.connect(srv_name, client.into_inner()).await?;

    Ok(NutClient::from(connection))
  }
}
