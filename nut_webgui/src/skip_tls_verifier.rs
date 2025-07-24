use nut_webgui_upsmc::rustls::{
  DigitallySignedStruct, Error, SignatureScheme,
  client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier},
  pki_types::{CertificateDer, ServerName, UnixTime},
};

#[derive(Debug)]
pub struct SkipTlsVerification;

impl ServerCertVerifier for SkipTlsVerification {
  fn verify_server_cert(
    &self,
    _end_entity: &CertificateDer<'_>,
    _intermediates: &[CertificateDer<'_>],
    _server_name: &ServerName<'_>,
    _ocsp: &[u8],
    _now: UnixTime,
  ) -> Result<ServerCertVerified, Error> {
    Ok(ServerCertVerified::assertion())
  }

  fn verify_tls12_signature(
    &self,
    _message: &[u8],
    _cert: &CertificateDer<'_>,
    _dss: &DigitallySignedStruct,
  ) -> Result<HandshakeSignatureValid, Error> {
    Ok(HandshakeSignatureValid::assertion())
  }

  fn verify_tls13_signature(
    &self,
    _message: &[u8],
    _cert: &CertificateDer<'_>,
    _dss: &DigitallySignedStruct,
  ) -> Result<HandshakeSignatureValid, Error> {
    Ok(HandshakeSignatureValid::assertion())
  }

  fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
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
