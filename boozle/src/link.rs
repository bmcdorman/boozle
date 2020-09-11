use super::channel;
use super::message::Msg;
use bus::BusReader;
use std::sync::mpsc;

pub enum Blob {}

pub enum DispatchCtl {}

pub enum LinkCtl {}

pub struct Dispatcher {}

pub struct Link {
  link: channel::Side<Msg<Blob>, Msg<Blob>>,
  dispatcher: channel::Side<DispatchCtl, LinkCtl>,
}

impl Link {}
