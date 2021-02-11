#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;
// use std::borrow::Borrow;
use rocket::http::RawStr;
use serde::{ Serialize, Deserialize };
use rocket_contrib::json::Json;
use rand::Rng;
use rocket::State;
// use std::sync::{Arc, Mutex};

#[get("/")]
fn index() -> &'static str {
    "Distributed Key Value Store"
}

#[derive(Deserialize, Debug, Serialize)]
struct Kvs {
    value: String
}

#[derive(Deserialize, Debug, Serialize)]
struct PutResult {
    message: String,
    replaced: bool
}

#[put("/kvs/<key>", data = "<kvs>")]
fn put_kvs(key: &RawStr, kvs:Json<Kvs>, state: State<AppState>) -> Json<PutResult> {
    let client = reqwest::blocking::Client::new();
    let url = format!("{}/kvs/keys/{}", state.choose_address(key), key);
    println!("request: {}:{} to {}", key, kvs.value, url);
    let response = client.put(&url[..])
        .json(&serde_json::json!({
            "value":kvs.value
        }))
        .send().unwrap()
        .json().unwrap();
    println!("response: {:?}", response);
    Json(response)
}

#[get("/kvs/<key>")]
fn get_kvs(key: &RawStr, state: State<AppState>) -> Json<Kvs> {
    state.print_view();
    let client = reqwest::blocking::Client::new();
    let url = format!("{}/kvs/keys/{}", state.choose_address(key), key);
    println!("url:{}", url);
    let response:Kvs = client.get(&url[..])
        .send().unwrap()
        .json().unwrap();
    println!("response: {:?}", response);
    Json(response)
}

#[get("/kvs/key-count")]
fn get_key_count(state: State<AppState>) -> Json<Kvs> {
    let client = reqwest::blocking::Client::new();
    let url = format!("{}/key_count", state.random_address());
    let response:Kvs = client.get(&url[..])
        .send().unwrap()
        .json().unwrap();
    Json(response)
}

#[get("/kvs/shards")]
fn get_shards(state: State<AppState>) -> Json<Kvs> {
    state.print_view();
    let client = reqwest::blocking::Client::new();
    let url = format!("{}/shards", state.random_address());
    let response:Kvs = client.get(&url[..])
        .send().unwrap()
        .json().unwrap();
    Json(response)
}

#[delete("/kvs/<key>")]
fn delete_kvs(key: &RawStr, state: State<AppState>) -> Json<Kvs> {
    let client = reqwest::blocking::Client::new();
    let url = format!("{}/kvs/keys/{}", state.choose_address(key), key);
    let response:Kvs = client.delete(&url[..])
        .send().unwrap()
        .json().unwrap();
    println!("response: {:?}", response);
    Json(response)
}

#[derive(Copy, Clone)]
struct AppState {
    pub repl_factor:u8,
    pub view:[IPAddress; 8],
    pub length:usize
}

#[derive(Copy, Clone, Default)]
struct IPAddress {
    pub ip:[u8;4],
    pub port:u32,
}

impl IPAddress {
    fn to_string(self) -> String{
        return format!("{}.{}.{}.{}:{}", self.ip[0], self.ip[1], self.ip[2], self.ip[3], self.port);
    }
}

fn build_ip(address:String) -> IPAddress {
    let split = address.split(":").collect::<Vec<&str>>();
    let ip_str = split[0];
    let port_str = split[1];
    let mut ip_address = IPAddress::default();
    for (i, v) in ip_str.split(".").enumerate() {
        ip_address.ip[i] = v.parse::<u8>().unwrap();
    }
    ip_address.port = port_str.parse::<u32>().unwrap();
    return ip_address;   
}

impl Default for AppState {
    fn default() -> Self {
        let mut envior = std::env::var("VIEW").unwrap();
        let view_iter = envior.split(",");
        let mut view = [IPAddress::default(); 8];
        let mut length:usize = 0;
        for (i,address) in view_iter.enumerate() {
            view[i] = build_ip(address.to_string());
            length += 1;
        }
        return AppState {
            repl_factor: std::env::var("REPL_FACTOR").unwrap()
                            .parse::<u8>().unwrap(),
            view: view,
            length:length
        };
    }
}

impl AppState {
    fn choose_address(self, _key:&RawStr) -> String {
        let mut rng = rand::thread_rng();
        let i:usize = rng.gen_range(0..self.length);
        return format!("http://localhost:{}", self.view[i].port)
    }

    fn print_view(self) {
        for i in 0..self.length {
            println!("{}", format!("http://localhost:{}", self.view[i].port))
        }
    }

    fn random_address(self) -> String {
        let mut rng = rand::thread_rng();
        let i:usize = rng.gen_range(0..self.length);
        return format!("http://localhost:{}", self.view[i].port);
    }
}

fn main() {
    let app_state = AppState::default();
    rocket::ignite()
        .manage(app_state)
        .mount("/", routes![index, put_kvs, get_kvs])//get_key_count, get_shards
        .launch();
}
