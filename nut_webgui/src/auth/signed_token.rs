use super::BinaryToken;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::marker::PhantomData;
use std::sync::Arc;

#[derive(Clone)]
pub struct SignedToken<T>
where
  T: BinaryToken,
{
  server_key: Arc<[u8]>,
  _token: PhantomData<T>,
}

pub enum SignatureError<T>
where
  T: BinaryToken,
{
  InvalidLength,
  InvalidSignature,
  TokenError(T::Error),
}

impl<T> SignedToken<T>
where
  T: BinaryToken,
{
  const KEY_SIZE: usize = 32;

  pub fn new(key: &[u8]) -> Self {
    Self {
      server_key: Arc::from(key),
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
    if bytes.len() <= Self::KEY_SIZE {
      return Err(SignatureError::InvalidLength);
    }

    let (payload, signature) = bytes.split_at(bytes.len() - Self::KEY_SIZE);

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
