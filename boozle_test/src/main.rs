use actix::prelude::*;
use boozle::pool::{Pool, InsertResolved, InsertUnresolved, Resolve, Route};
use tokio::time::delay_for;

#[derive(Message, Debug)]
#[rtype(result = "Result<TestResponse, String>")]
struct Test {

}

#[derive(Debug)]
struct TestResponse {

}

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


#[actix_rt::main]
async fn main() {
  let a = (A {}).start();
  let pool = Pool::new().start();
  
  println!("{:?}", pool.send(InsertUnresolved {
    id: 0
  }).await);

  let pool2 = pool.clone();

  actix::spawn(async move {
    delay_for(std::time::Duration::from_secs(10)).await;
    let _ = pool2.send(Resolve {
      id: 0,
      recipient: (A {}).start().recipient()
    }).await;
  });
  
  println!("{:?}", pool.send(Route {
    id: 0,
    message: Test {}
  }).await);
  println!("{:?}", pool.send(Route {
    id: 0,
    message: Test {}
  }).await);
  
}
