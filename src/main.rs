#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;
extern crate crypto;
mod app_state;
mod routes;


fn main() { 
    let app_state = app_state::SharedState::default();
    rocket::ignite()
        .manage(app_state)
        .mount("/", routes![routes::index, routes::put_kvs, routes::get_kvs, routes::delete_kvs, routes::get_key_count, routes::get_shards, routes::view_change])
        .launch();
}

// extern crate openssl;

// use openssl::rsa::{Rsa, Padding};
// use openssl::symm::Cipher;

// fn main() {
//     let passphrase = "boobies_are_cool";

//     let rsa = Rsa::generate(1024).unwrap();
//     let private_key: Vec<u8> = rsa.private_key_to_pem_passphrase(Cipher::aes_128_cbc(), passphrase.as_bytes()).unwrap();
//     let public_key: Vec<u8> = rsa.public_key_to_pem().unwrap();

//     println!("Private key: {}", String::from_utf8(private_key).unwrap());
//     println!("Public key: {}", String::from_utf8(public_key).unwrap());
// }