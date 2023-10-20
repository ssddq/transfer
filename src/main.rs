mod args;
mod receive;
mod send;
mod tcp;

use clap::Parser;
use crate::args::Command::*;
use crate::receive::*;
use crate::send::*;


fn main() 
{ let args = args::CliArgs::parse();

  match args.command
  { Receive(a) => receive(a).unwrap(),
    Send(a) => send(a).unwrap(),
  }
}
