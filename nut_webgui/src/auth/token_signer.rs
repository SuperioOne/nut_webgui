use super::{BinaryToken, error::SignatureError};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::marker::PhantomData;

#[derive(Clone)]
pub struct TokenSigner<K, T>
where
  T: BinaryToken,
  K: AsRef<[u8]>,
{
  server_key: K,
  _token: PhantomData<T>,
}

impl<K, T> TokenSigner<K, T>
where
  T: BinaryToken,
  K: AsRef<[u8]>,
{
  const SIGNATURE_SIZE: usize = 32;

  pub fn new(key: K) -> Self {
    Self {
      server_key: key,
      _token: PhantomData::default(),
    }
  }

  pub fn sign_token(&self, token: &T) -> Vec<u8> {
    let mut hmac =
      Hmac::<Sha256>::new_from_slice(self.server_key.as_ref()).expect("infallible: hmac key size");

    let mut signed = token.as_bytes();

    hmac.update(&signed);
    let signature = hmac.finalize();

    signed.extend_from_slice(signature.into_bytes().as_slice());
    signed
  }

  pub fn from_bytes(&self, bytes: &[u8]) -> Result<T, SignatureError<T>> {
    if bytes.len() <= Self::SIGNATURE_SIZE {
      return Err(SignatureError::InvalidLength);
    }

    let (payload, signature) = bytes.split_at(bytes.len() - Self::SIGNATURE_SIZE);

    let mut hmac =
      Hmac::<Sha256>::new_from_slice(self.server_key.as_ref()).expect("infallible: hmac key size");

    hmac.update(payload);
    hmac
      .verify_slice(signature)
      .map_err(|_| SignatureError::InvalidSignature)?;

    let token = T::from_bytes(payload).map_err(|err| SignatureError::TokenError(err))?;

    Ok(token)
  }
}
