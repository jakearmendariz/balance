extern crate crypto;
use rocket::http::RawStr;
use serde::{ Serialize, Deserialize };
use rocket_contrib::json::Json;
use rocket::State;
use crate::app_state::SharedState;

use std::borrow::BorrowMut;

#[get("/")]
pub fn index() -> &'static str {
    "Distributed Key Value Store"
}

#[derive(Deserialize, Debug, Serialize)]
pub struct Kvs {
    value: String
}

#[derive(Deserialize, Debug, Serialize)]
pub struct PutResult {
    message: String,
    replaced: bool
}

#[derive(Deserialize, Debug, Serialize)]
pub struct ViewChange {
    view: String,
    repl_factor: u8
}

#[derive(Deserialize, Debug, Serialize)]
pub struct ViewChangeResult {
    message: String,
    shards: Vec<String>
}

#[put("/kvs/keys/view-change", data = "<view_change>")]
pub fn view_change(view_change:Json<ViewChange>, shared_state: State<SharedState>) -> Json<ViewChangeResult> {
    let mut app_state = shared_state.state.lock().expect("lock shared data");
    app_state.repl_factor = view_change.repl_factor;
    let view_iter = view_change.view.split(",");
    app_state.length = 0;
    for (i,address) in view_iter.enumerate() {
        app_state.borrow_mut().build_ip(address.to_string(), i);
        let ip_address = app_state.view[i];
        app_state.build_ring(ip_address.to_string(), i);
    }
    app_state.ring.sort_by(|a, b| a.hash.cmp(&b.hash));
    let url = format!("{}/kvs/view-change", app_state.random_address());
    println!("url:{}", url);
    let client = reqwest::blocking::Client::new();
    let response = client.put(&url[..])
        .json(&serde_json::json!({
            "view":view_change.view,
            "repl-factor":view_change.repl_factor
        }))
        .send().unwrap()
        .json().unwrap();
    println!("response:{:?}", response);
    Json(response)
}

#[put("/kvs/<key>", data = "<kvs>")]
pub fn put_kvs(key: &RawStr, kvs:Json<Kvs>, shared_state: State<SharedState>) -> Json<PutResult> {
    let state = shared_state.state.lock().expect("lock shared data");
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
pub fn get_kvs(key: &RawStr, shared_state: State<SharedState>) -> Json<Kvs> {
    let state = shared_state.state.lock().expect("lock shared data");
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
pub fn get_key_count(shared_state: State<SharedState>) -> Json<Kvs> {
    let state = shared_state.state.lock().expect("lock shared data");
    let client = reqwest::blocking::Client::new();
    let url = format!("{}/key_count", state.random_address());
    let response:Kvs = client.get(&url[..])
        .send().unwrap()
        .json().unwrap();
    Json(response)
}

#[get("/kvs/shards")]
pub fn get_shards(shared_state: State<SharedState>) -> Json<Kvs> {
    let state = shared_state.state.lock().expect("lock shared data");
    state.print_view();
    let client = reqwest::blocking::Client::new();
    let url = format!("{}/shards", state.random_address());
    let response:Kvs = client.get(&url[..])
        .send().unwrap()
        .json().unwrap();
    Json(response)
}

#[delete("/kvs/<key>")]
pub fn delete_kvs(key: &RawStr, shared_state: State<SharedState>) -> Json<Kvs> {
    let state = shared_state.state.lock().expect("lock shared data");
    let client = reqwest::blocking::Client::new();
    let url = format!("{}/kvs/keys/{}", state.choose_address(key), key);
    let response:Kvs = client.delete(&url[..])
        .send().unwrap()
        .json().unwrap();
    println!("response: {:?}", response);
    Json(response)
}
