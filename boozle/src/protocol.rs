use serde::Serialize;

use bytes::Buf;

pub fn write<T: Serialize>(value: &T) -> Vec<u8> {}

pub fn try_read<T: Deserialize>(buf: &[u8]) -> Option<(usize, T)> {
  let a = buf[0];
}
