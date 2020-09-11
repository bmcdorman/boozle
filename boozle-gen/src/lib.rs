use boozle_parser::{parse, Unit};
use std::collections::HashMap;
use std::io::{Error, Read};

use std::fs::File;

use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum Entry<T> {
  Directory(Directory<T>),
  File(T),
}

impl<T> Entry<T> {
  pub fn map<O>(&self, f: fn(value: &T) -> O) -> Entry<O> {
    match self {
      Self::File(value) => Entry::File(f(value)),
      Self::Directory(dir) => Entry::Directory(Directory(
        dir
          .0
          .iter()
          .map(|(key, value)| (key.clone(), value.map(f)))
          .collect(),
      )),
    }
  }
}

#[derive(Debug)]
pub struct Directory<T>(HashMap<String, Entry<T>>);

pub struct Forest<T> {
  roots: Vec<Tree<T>>,
}

impl<T> Forest<T> {
  pub fn new<I: IntoIterator<Item = Tree<T>>>(roots: I) -> Self {
    Self {
      roots: roots.into_iter().collect(),
    }
  }

  pub fn iter(&self) -> impl Iterator<Item = &Tree<T>> {
    self.roots.iter()
  }

  pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Tree<T>> {
    self.roots.iter_mut()
  }
}

impl<T> IntoIterator for Forest<T> {
  type Item = Tree<T>;
  type IntoIter = <Vec<Tree<T>> as IntoIterator>::IntoIter;

  fn into_iter(self) -> Self::IntoIter {
    self.roots.into_iter()
  }
}

pub struct Tree<T> {
  root: T,
  children: Vec<Tree<T>>,
}

impl<T> Tree<T> {
  pub fn leaf(root: T) -> Self {
    Self {
      root,
      children: Vec::new(),
    }
  }

  pub fn new<C: IntoIterator<Item = Tree<T>>>(root: T, children: C) -> Self {
    Self {
      root,
      children: children.into_iter().collect(),
    }
  }

  pub fn map<O>(&self, f: fn(&T) -> O) -> Tree<O> {
    Tree {
      root: f(&self.root),
      children: self.children.iter().map(|child| child.map(f)).collect(),
    }
  }

  pub fn map_into<O>(self, f: fn(T) -> O) -> Tree<O> {
    Tree {
      root: f(self.root),
      children: self
        .children
        .into_iter()
        .map(|child| child.map_into(f))
        .collect(),
    }
  }
}

fn submods(unit: Unit) -> Vec<PathBuf> {
  unit.
}

fn load_units<I>(paths: I) -> Result<Forest<Unit>, Error>
where
  I: IntoIterator,
  I::Item: AsRef<Path>,
{
  for path in paths.into_iter() {
    let file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents);
    let file_stem = path.file_stem().unwrap().to_str().unwrap();
    let parse = boozle_parser::parse(file_stem, contents.as_str()).unwrap();
    
  }
  Forest::new(vec![])
}
