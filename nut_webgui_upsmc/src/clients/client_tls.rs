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

// let client_conf = ClientConfig::builder()
//   .with_platform_verifier()
//   .with_no_client_auth();

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

#[cfg(test)]
mod test {
  use crate::clients::{AsyncNutClient, NutClient};
  use crate::rustls::{
    DigitallySignedStruct,
    client::{
      WebPkiServerVerifier,
      danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier},
    },
    pki_types::{CertificateDer, ServerName, UnixTime},
  };
  use rustls_platform_verifier::BuilderVerifierExt;
  use std::sync::Arc;
  use tokio_rustls::rustls::{ClientConfig, SignatureScheme};

  #[derive(Debug)]
  pub struct SkipVerification;

  impl ServerCertVerifier for SkipVerification {
    fn verify_server_cert(
      &self,
      _end_entity: &CertificateDer<'_>,
      _intermediates: &[CertificateDer<'_>],
      _server_name: &ServerName<'_>,
      _ocsp: &[u8],
      _now: UnixTime,
    ) -> Result<ServerCertVerified, crate::rustls::Error> {
      Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
      &self,
      message: &[u8],
      cert: &CertificateDer<'_>,
      dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, tokio_rustls::rustls::Error> {
      Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
      &self,
      message: &[u8],
      cert: &CertificateDer<'_>,
      dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, tokio_rustls::rustls::Error> {
      Ok(HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<tokio_rustls::rustls::SignatureScheme> {
      vec![
        SignatureScheme::RSA_PKCS1_SHA1,
        SignatureScheme::ECDSA_SHA1_Legacy,
        SignatureScheme::RSA_PKCS1_SHA256,
        SignatureScheme::ECDSA_NISTP256_SHA256,
        SignatureScheme::RSA_PKCS1_SHA384,
        SignatureScheme::ECDSA_NISTP384_SHA384,
        SignatureScheme::RSA_PKCS1_SHA512,
        SignatureScheme::ECDSA_NISTP521_SHA512,
        SignatureScheme::RSA_PSS_SHA256,
        SignatureScheme::RSA_PSS_SHA384,
        SignatureScheme::RSA_PSS_SHA512,
        SignatureScheme::ED25519,
        SignatureScheme::ED448,
      ]
    }
  }

  // #[tokio::test]
  async fn connect_tls() {
    let verifier = Arc::new(SkipVerification);
    let mut client_conf = ClientConfig::builder()
      .with_platform_verifier()
      .unwrap()
      .with_no_client_auth();

    client_conf.dangerous().set_certificate_verifier(verifier);

    let client = NutClient::connect_with_tls(
      "localhost:3493",
      ServerName::try_from("localhost").unwrap(),
      Arc::new(client_conf),
    );

    let result = client
      .await
      .inspect_err(|e| println!("conn {}", e))
      .unwrap()
      .list_ups()
      .await
      .inspect_err(|e| println!("list {}", e))
      .unwrap();

    println!("{:?}", result);
  }
}
