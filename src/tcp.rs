use aes_gcm::aead;
use aes_gcm::{Aes256Gcm, KeyInit };
use dialoguer::Confirm;
use hkdf::Hkdf;
use p384::ecdh::{SharedSecret, EphemeralSecret};
use p384::{EncodedPoint, PublicKey};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};

// Open a TCP/IP connection with the device at the given IP address and remote port.

pub fn connect_to(ip: String, port: u16) -> (TcpStream, SharedSecret)
{ let addr = SocketAddr::new(ip.parse().expect("Failed to parse IP address"), port);
  let stream = TcpStream::connect(addr).expect("Connection failed.");

  println!("Established connection with {}.\n", addr);
  
  return handshake_sender(stream);
}

// Listen for a connection on the given port.

#[tokio::main]
pub async fn listen_on(port: u16) -> (TcpStream, SharedSecret)
{ let addr = SocketAddr::new("0.0.0.0".parse().expect("Failed to parse IP address"), port);
  let listener = TcpListener::bind(addr).unwrap();

  let port = listener.local_addr().unwrap().port();

  println!("Waiting for a connection on port {}.\n", port);

  let public_ip = public_ip::addr().await.unwrap();
  let local_ip = local_ip_address::local_ip().unwrap();

  println!("The local IP address of this device is {}.\n", local_ip);

  println!("You can initiate a connection from another device on the same network by running:\n");
  println!("    transfer send {} {} <path>\n", local_ip, port);
  
  println!("If your router is set up to forward port {} to this device, you can also use this device's public IP address:\n", port);
  println!("    transfer send {} {} <path>\n", public_ip, port);
  println!("Note: In either case, you will need to ensure that this device does not have a firewall blocking incoming TCP connections.\n");

  match listener.accept()
  { Ok((stream, addr)) => 
    { println!("\nEstablished connection with {}.\n", addr);
      return handshake_receiver(stream); 
    },
    Err(e) => 
    { eprintln!("Error: {:#?}", e);
      std::process::exit(1)
    }
  }
}

// Process the handshake result as the sending device.

fn handshake_sender(stream: TcpStream) -> (TcpStream, SharedSecret)
{ let result = handshake(stream);

  let stream = result.0;
  let keys = result.1;

  println!("The sender's public key is:\n\n{}\n", EncodedPoint::from(keys.local_key.public_key()));
  println!("The receiver's public key is:\n\n{}\n", EncodedPoint::from(keys.remote_public_key));

  request_key_confirmation();

  let shared_secret = keys.local_key.diffie_hellman(&keys.remote_public_key);

  return (stream, shared_secret);
}

// Process the handshake result as the receiving device.

fn handshake_receiver(stream: TcpStream) -> (TcpStream, SharedSecret)
{ let result = handshake(stream);

  let stream = result.0;
  let keys = result.1;

  println!("The sender's public key is:\n\n{}\n", EncodedPoint::from(keys.remote_public_key));
  println!("The receiver's public key is:\n\n{}\n", EncodedPoint::from(keys.local_key.public_key()));

  request_key_confirmation();

  let shared_secret = keys.local_key.diffie_hellman(&keys.remote_public_key);

  return (stream, shared_secret);
}

// Prompt the user to confirm the keys match.

fn request_key_confirmation()
{ let confirmation = Confirm::new().with_prompt("Is this pair of keys identical on both devices?").interact().unwrap();
 
  if !confirmation
  { println!("\nHandshake failed. Aborting.");
    std::process::exit(1)
  }
  else
  { println!("\n");
    return;
  }
}

// Perform a key-exchange over the given TCP connection.

fn handshake(mut stream: TcpStream) -> (TcpStream, Keys)
{ let mut rng = &mut aead::OsRng;
  let local_key = EphemeralSecret::random(&mut rng);
  let local_public_key = EncodedPoint::from(local_key.public_key());

  let mut remote_public_key_buffer: [u8; 97] = [0; 97];
  
  stream.write(local_public_key.as_ref()).expect("Communication error");
  stream.read(&mut remote_public_key_buffer).expect("Communication error");

  let remote_public_key = PublicKey::from_sec1_bytes(&remote_public_key_buffer).expect("Error parsing received public key.");

  let keys = Keys { local_key, remote_public_key };

  return (stream, keys);
}

// Produce the encryption cipher based on a shared secret.

pub fn compute_cipher(secret: SharedSecret) -> Aes256Gcm
{ let key = Hkdf::<sha2::Sha256>::extract(None, secret.raw_secret_bytes()).0;

  return Aes256Gcm::new(&key);
}

// Struct to hold the local and remote keys.

struct Keys
{ local_key: EphemeralSecret,
  remote_public_key: PublicKey,
}
