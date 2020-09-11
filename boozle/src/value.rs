use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Value {
  Lit(Box<[u8]>),
  Ref { owner_id: u64, actor_id: u64 },
}
