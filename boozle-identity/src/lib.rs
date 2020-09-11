use openssl::hash::MessageDigest;
use openssl::rsa::Rsa;
use openssl::sign::{Signer, Verifier};

use openssl::bn::BigNum;
pub use openssl::dsa::Dsa;
pub use openssl::error::ErrorStack;
pub use openssl::pkey::{HasPublic, Id, PKey, Private, Public};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

struct PublicKeyVisitor;

impl<'de> serde::de::Visitor<'de> for PublicKeyVisitor {
  type Value = PKey<Public>;
  fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(formatter, "Failed to extract public key from PEM")
  }
  fn visit_bytes<E: serde::de::Error>(self, b: &[u8]) -> Result<Self::Value, E> {
    match PKey::public_key_from_pem(b) {
      Ok(public_key) => Ok(public_key),
      Err(_) => Err(serde::de::Error::invalid_value(
        serde::de::Unexpected::Bytes(b),
        &self,
      )),
    }
  }
}

struct PrivateKeyVisitor;

impl<'de> serde::de::Visitor<'de> for PrivateKeyVisitor {
  type Value = PKey<Private>;
  fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(formatter, "Failed to extract private key from PEM")
  }
  fn visit_bytes<E: serde::de::Error>(self, b: &[u8]) -> Result<Self::Value, E> {
    match PKey::private_key_from_pem(b) {
      Ok(private_key) => Ok(private_key),
      Err(_) => Err(serde::de::Error::invalid_value(
        serde::de::Unexpected::Bytes(b),
        &self,
      )),
    }
  }
}

pub struct Identity<T: HasPublic> {
  key: PKey<T>,
}

impl<T: HasPublic> Identity<T> {
  pub fn new(key: PKey<T>) -> Self {
    Self { key }
  }

  /* pub fn encrypt(&self, data: &[u8]) -> Result<Box<[u8]>, ErrorStack> {
      let mut buf = Box::new([0u8; self.key.size()]);
      match self.key.id() {
          Id::RSA => {
              let rsa = self.key.rsa()?;
              rsa.public_encrypt()
          }
          _ => {}
      }

      Ok(Vec::new().into_boxed_slice())
  }*/
}

impl Identity<Public> {
  pub fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool, ErrorStack> {
    let mut verifier = Verifier::new(MessageDigest::sha256(), &self.key)?;
    verifier.update(data)?;
    verifier.verify(signature)
  }
}

impl Identity<Private> {
  pub fn generate_rsa(bits: u32) -> Result<Self, ErrorStack> {
    let private_key = Rsa::<Private>::generate(bits)?;

    Ok(Self::new(PKey::from_rsa(private_key)?))
  }

  pub fn sign(&self, data: &[u8]) -> Result<Box<[u8]>, ErrorStack> {
    let mut signer = Signer::new(MessageDigest::sha256(), &self.key)?;

    signer.update(data)?;

    let signature = signer.sign_to_vec()?;
    Ok(signature.into_boxed_slice())
  }

  pub fn public(&self) -> Result<Option<Identity<Public>>, ErrorStack> {
    match self.key.id() {
      Id::RSA => {
        let rsa = self.key.rsa()?;
        Ok(Some(Identity::<Public> {
          key: PKey::from_rsa(Rsa::from_public_components(
            BigNum::from_slice(rsa.n().to_vec().as_slice())?,
            BigNum::from_slice(rsa.e().to_vec().as_slice())?,
          )?)?,
        }))
      }
      _ => Ok(None),
    }
  }
}

impl Serialize for Identity<Public> {
  fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_bytes(
      self
        .key
        .public_key_to_pem()
        .map_err(|_| serde::ser::Error::custom("Failed to generate PEM for public key"))?
        .as_slice(),
    )
  }
}

impl Serialize for Identity<Private> {
  fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_bytes(
      self
        .key
        .private_key_to_pem_pkcs8()
        .map_err(|_| serde::ser::Error::custom("Failed to generate PEM for private key"))?
        .as_slice(),
    )
  }
}

impl<'a> Deserialize<'a> for Identity<Public> {
  fn deserialize<D: Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
    let key = deserializer.deserialize_bytes(PublicKeyVisitor {})?;
    Ok(Self { key })
  }
}

impl<'a> Deserialize<'a> for Identity<Private> {
  fn deserialize<D: Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
    let key = deserializer.deserialize_bytes(PrivateKeyVisitor {})?;
    Ok(Self { key })
  }
}
