use actix::prelude::*;
use boozle::pool::{Pool, PoolHelpers, InsertResolved, InsertUnresolved, Resolve, Get};
use boozle::object::{Object, Call, ProxyInfo, ObjectActorHelpers, CallMut, Return, CallError, ObjectActor};
use boozle::value::{LocalValue};
use boozle::connection::{Connection, ConnectionHelpers, req, res};
use tokio::time::delay_for;
use async_trait;

#[derive(Message, Debug)]
#[rtype(result = "Result<TestResponse, String>")]
struct Test {

}

#[derive(Debug)]
struct TestResponse {

}

#[derive(Debug)]
struct A {

}

impl Actor for A {
  type Context = Context<Self>;
}

impl Handler<Test> for A {
  type Result = Result<TestResponse, String>;

  fn handle(&mut self, test: Test, ctx: &mut Context<Self>) -> Self::Result {
    println!("Hello, world!");
    Ok(TestResponse {})
  }
}

#[derive(Debug)]
struct TestClass {

}

#[async_trait::async_trait]
impl Object for TestClass {
  async fn call(&self, call: Call) -> Result<Return, CallError> {
    match call.method_id {
      0 => {
        match call.argument {
          None => {
            panic!("asd");
          },
          Some(arg) => {
            match arg {
              LocalValue::Lit(lit) => {
                let arg: u64 = bincode::deserialize(&lit).unwrap();
                // println!("Call {:?}!", arg);
              },
              _ => panic!("asd")
            }
          }
        }
        
        Ok(Return {
          result: Some(LocalValue::Lit(bincode::serialize(&1u64).unwrap().into_boxed_slice().into()))
        })
      },
      _ => Err(CallError::UnknownMethod)
    }

    
  }

  async fn call_mut(&mut self, call: CallMut) -> Result<Return, CallError> {
    unimplemented!();
  }

  fn proxy_info(&self) -> Option<ProxyInfo> {
      None
  }
}

#[actix_rt::main]
async fn main() {
  let (tx1, rx1) = tokio::sync::mpsc::channel(100);
  let (tx2, rx2) = tokio::sync::mpsc::channel(100);
  let left = Connection::new(tx1, rx2).start();
  let right = Connection::new(tx2, rx1).start();
  
  let key = left.pool().await.expose(LocalValue::from_object(TestClass {})).await.unwrap().key;
  let mut val = 0u64;
  let start = std::time::SystemTime::now();
  let measure_duration = std::time::Duration::from_secs(5);
  while std::time::SystemTime::now().duration_since(start).unwrap() < measure_duration {
    
    right.req(req::Req::Call(req::Call {
      object_id: key,
      argument: Some(LocalValue::from_lit(&val)),
      method_id: 0,
      mutable: false,
      store_result: true
    })).await.unwrap();
    val += 1;
  }

  println!("perf: {}/s", val as f64 / measure_duration.as_secs_f64())

  
}
