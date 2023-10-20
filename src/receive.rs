use aes_gcm::{Aes256Gcm, Nonce, aead::{Aead, Payload}};
use anyhow::{Result};
use crate::args::*;
use crate::tcp::*;
use generic_array::{GenericArray, typenum::U12};
use indicatif::{ProgressStyle, ProgressBar};
use std::io::{Read, Cursor};
use std::net::TcpStream;
use zip::{ZipArchive};

#[derive(Debug)]
pub enum MessageReceiveError
{ Nonce,
  Size,
  Body,
  Decrypt,
  Unzip
}

// Wait for a TCP connection, 
// then receive, decrypt and extract 
// a single zipped and encrypted file or directory.

pub fn receive(args: ReceiveArgs) -> Result<(), MessageReceiveError>
{ let result = listen_on(args.port);

  let mut stream = result.0;
  let cipher = compute_cipher(result.1);

  let file = receive_message(&mut stream, cipher)?;

  unzip(file, args.path.as_str()).map_err(|_| MessageReceiveError::Unzip)?;
  
  println!("Files successfully written to {}. Terminating.", args.path);

  return Ok(());
}

// Receive and decrypt a message over the stream.

fn receive_message(stream: &mut TcpStream, cipher: Aes256Gcm) -> Result<Vec<u8>, MessageReceiveError>
{ let nonce = read_nonce(stream)?;
  let message_size: u64 = read_u64(stream)
    .map_err(|_| MessageReceiveError::Size)?;
  let message: Vec<u8> = read_bytes(stream, message_size)
    .map_err(|_| MessageReceiveError::Body)?;
  println!("size: {}", message_size);
  let decrypted = cipher
    .decrypt(&nonce, Payload::from(message.as_slice()))
    .map_err(|_| MessageReceiveError::Decrypt);

  return decrypted;
}

// Read the nonce from the socket.

fn read_nonce(stream: &mut TcpStream) -> Result<Nonce<U12>, MessageReceiveError>
{ let mut buffer: [u8; 12] = [0; 12];
  stream
    .read_exact(&mut buffer)
    .map_err(|_| MessageReceiveError::Nonce)?;

  return GenericArray::<u8, U12>::from_exact_iter(buffer)
    .ok_or(MessageReceiveError::Nonce);
}

// Read the given number of bytes from the socket.
//
// Generates a progress bar for the file transfer.

fn read_bytes(stream: &mut TcpStream, count: u64) -> Result<Vec<u8>>
{ let mut buffer: Vec<u8> = vec![0; count as usize]; 
  
  { let progress_bar = ProgressBar::new(count);
    let transfer_style = ProgressStyle::with_template("Transferring: received {bytes}/{total_bytes}")?;
    progress_bar.set_style(transfer_style);

    std::io::copy(stream, &mut progress_bar.wrap_write(Cursor::new(&mut buffer)))?;

    let completed_style = ProgressStyle::with_template("Transferring: received {total_bytes}.")?;
    progress_bar.set_style(completed_style);
    progress_bar.finish();
  }

  return Ok(buffer);
}

// Read a single u64 from the socket.

fn read_u64(stream: &mut TcpStream) -> Result<u64 >
{ let mut buffer: [u8; 8] = [0; 8];
  stream.read_exact(&mut buffer)?;

  return Ok(u64::from_be_bytes(buffer));
}

// Extract the unencrypted binary file to the given path.

fn unzip(input: Vec<u8>, path: &str) -> Result<()>
{ let mut zip = ZipArchive::new(Cursor::new(input))?;
  zip.extract(path)?;

  return Ok(());
}
