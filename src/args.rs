use clap::{Args, Parser, Subcommand};

/// Utility to transfer files between devices.
#[derive(Parser, Debug)]

#[command(author, version, about, long_about = None)]
pub struct CliArgs 
{ #[command(subcommand)]
  pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command
{ #[command(about="Receive a file on this device", name="receive")]
  Receive(ReceiveArgs),

  #[command(about="Send a file from this device", name="send")]
  Send(SendArgs),
}

#[derive(Args, Debug)]
pub struct SendArgs
{ /// Destination IP address
  #[clap(value_name = "ip")]
  pub ip: String,

  /// Destination port
  #[clap(value_name = "port")]
  pub port: u16,

 /// File or folder to transfer
 #[clap(value_name = "path")]
 pub path: String,
}

#[derive(Args, Debug)]
pub struct ReceiveArgs
{ /// Port to use.
  #[arg(short, long, default_value_t = 0)]
  pub port: u16,

  /// Output destination
  #[clap(value_name = "path")]
  pub path: String,
}
