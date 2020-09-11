#[macro_use]
extern crate lalrpop_util;

mod ast;

pub use ast::*;

lalrpop_mod!(pub boozle);

pub type ParseError<'a> = lalrpop_util::ParseError<usize, lalrpop_util::lexer::Token<'a>, &'a str>;

pub struct Unit {
  pub name: String,
  pub decls: Vec<Decl>,
}

pub fn parse<'a>(name: String, text: &'a str) -> Result<Unit, ParseError<'a>> {
  Ok(Unit {
    name,
    decls: boozle::UnitParser::new().parse(text)?,
  })
}
