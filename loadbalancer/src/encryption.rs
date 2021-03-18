extern crate lazy_static;
extern crate base64;
use openssl::rsa::{Rsa, Padding};
use base64::{encode, decode};

lazy_static::lazy_static! {
    static ref PRIVATE_KEY:String = std::fs::read_to_string("private.key")
        .expect("Couldn't read private.key");
}
lazy_static::lazy_static! {
    static ref PUBLIC_KEY:String = std::fs::read_to_string("public.key")
        .expect("Couldn't read public.key");
}

pub fn encrypt(data:String) -> String {
    // Encrypt with public key
    let rsa = Rsa::public_key_from_pem(PUBLIC_KEY.as_bytes()).unwrap();
    let mut buf: Vec<u8> = vec![0; rsa.size() as usize];
    let _ = rsa.public_encrypt(data.as_bytes(), &mut buf, Padding::PKCS1).unwrap();
    encode(&buf)
}

pub fn decrypt(data:String) -> String {
    let passphrase = std::env::var("PASSPHRASE").unwrap();
    // Decrypt with private key
    let hash = decode(data).unwrap();
    let rsa = Rsa::private_key_from_pem_passphrase(PRIVATE_KEY.as_bytes(), passphrase.as_bytes()).unwrap();
    let mut buf: Vec<u8> = vec![0; rsa.size() as usize];
    let _ = rsa.private_decrypt(&hash, &mut buf, Padding::PKCS1).unwrap();

    let decrypted_str = String::from_utf8(buf).unwrap();
    decrypted_str.chars().filter(|c| c.is_alphanumeric()).collect::<String>()
}