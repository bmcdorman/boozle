#[derive(Debug)]
pub struct Id {
  pub text: String,
}

#[derive(Debug)]
pub struct Mod {
  pub name: Id,
  pub decls: Option<Vec<Decl>>,
}

#[derive(Debug)]
pub struct Trait {
  pub name: Id,
  pub members: Vec<Fn>,
}

#[derive(Debug)]
pub struct Path {
  pub components: Vec<Id>,
}

#[derive(Debug)]
pub struct Type {
  pub path: Path,
}

#[derive(Debug)]
pub struct Param {
  pub name: Id,
  pub ty: Type,
}

#[derive(Debug)]
pub struct Fn {
  pub name: Id,
  pub params: Vec<Param>,
  pub result: Option<Type>,
}

#[derive(Debug)]
pub struct Svc {
  pub name: Id,
  pub ty: Type,
}

#[derive(Debug)]
pub struct Use {
  pub path: Path,
}

#[derive(Debug)]
pub enum Decl {
  Trait(Trait),
  Svc(Svc),
  Mod(Mod),
  Use(Use),
}
