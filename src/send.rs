use aes_gcm::{Nonce, Aes256Gcm, aead::{Aead, AeadCore, OsRng, Payload}};
use anyhow::{Result, Context};
use crate::args::*;
use crate::tcp::*;
use generic_array::typenum::U12;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs::{metadata};
use std::io::{Write, Cursor};
use std::net::TcpStream;
use zip::write::FileOptions;
use zip::ZipWriter;

#[derive(Debug)]
pub enum MessageSendError
{ Zip,
  Encrypt,
  Nonce,
  Size,
  Body,
}

// Connect to a remote device over TCP,
// then encrypt, zip and transfer a file or directory.

pub fn send(args: SendArgs) -> Result<(), MessageSendError>
{ let result = connect_to(args.ip, args.port);

  let mut stream = result.0;
  let cipher = compute_cipher(result.1);

  let message = zip_path(args.path.as_str())
    .map_err(|_| MessageSendError::Zip)?;
  send_message(&mut stream, cipher, message)?;

  return Ok(());
}

// Encrypt the message with the given cipher and a randomly generated nonce,
// then send the encrypted message over the TcpStream.
//
// Generates a progress bar for the file transfer.

fn send_message(stream: &mut TcpStream, cipher: Aes256Gcm, message: Vec<u8>) -> Result<(), MessageSendError>
{ let nonce: Nonce<U12> = Aes256Gcm::generate_nonce(&mut OsRng);
  let encrypted: Vec<u8> = cipher
    .encrypt(&nonce, Payload::from(message.as_slice()))
    .map_err(|_| MessageSendError::Encrypt)?;
  let message_size: u64 = encrypted.len() as u64;

  stream
    .write(&nonce)
    .map_err(|_| MessageSendError::Nonce)?;
  stream
    .write(&message_size.to_be_bytes())
    .map_err(|_| MessageSendError::Size)?;

  // Send encrypted message with progress bar. 
  { let progress_bar = ProgressBar::new(message_size);
    let transfer_style = ProgressStyle::with_template("Transferring: sent {bytes}/{total_bytes}")
          .unwrap();
    progress_bar.set_style(transfer_style);

    std::io::copy(&mut Cursor::new(encrypted), &mut progress_bar.wrap_write(stream))
      .map_err(|_| MessageSendError::Body)?;

    let completed_style = ProgressStyle::with_template("Transferring: sent {total_bytes}.")
      .unwrap();
    progress_bar.set_style(completed_style);
    progress_bar.finish();
  }

  return Ok(());
}

// Zip the given path. 

fn zip_path(path: &str) -> Result<Vec<u8>>
{ let buffer: Vec<u8> = vec![];
  let cursor = Cursor::new(buffer);
  let options = FileOptions::default()
    .compression_method(zip::CompressionMethod::Stored);
  let metadata = metadata(path)?;

  let mut zip = ZipWriter::new(cursor);

  let progress_bar = ProgressBar::new(1);
  let style = ProgressStyle::with_template("Creating zip file: {wide_msg}")
        .unwrap();
  progress_bar.set_style(style);

  if metadata.is_file()
  { let path_name = std::path::Path::new(path)
      .to_str()
      .context("Invalid path.")?;
    progress_bar.set_message(format!("{path_name}"));
    zip.start_file(path_name, options)?;

    let contents = std::fs::read(path)
      .context("Failed to read path")?;
    zip.write(&contents)?;

    let finished = zip.finish()?;
    progress_bar.finish_with_message("done.");
    return Ok(finished.into_inner());
  }
  else if metadata.is_dir()
  { for p in walkdir::WalkDir::new(path)
    { let subpath = p?;

      let metadata = subpath.metadata()?;
      let subpath_name = subpath
          .path()
          .to_str()
          .context("Invalid path.")?;

      if metadata.is_file()
      { progress_bar.set_message(format!("{subpath_name}"));
        zip.start_file(subpath_name, options)?;

        let contents = std::fs::read(subpath.path())
          .context("Failed to read path")?;
        zip.write(&contents)?;
      }
      else if metadata.is_dir()
      { zip.add_directory(subpath_name, options)?; }
    }

    let finished = zip.finish()?;
    progress_bar.finish_with_message("done.");
    return Ok(finished.into_inner());
  }
  else
  { println!("Error: path is not a file or directory. Aborting.");
    std::process::exit(1);
  }
}
