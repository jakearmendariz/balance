extern crate crypto;
use rocket::http::RawStr;
use serde::{ Serialize, Deserialize };
use rocket_contrib::json::Json;
use rocket::State;
use crate::app_state::AppState;
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
    shards: u8
}

#[put("/kvs/view-change", data = "<view_change>")]
pub fn view_change(view_change:Json<ViewChange>, app_state: State<AppState>) -> Json<ViewChangeResult> {
    app_state.repl_factor = view_change.repl_factor;
    let view_iter = view_change.view.split(",");
    app_state.length = 0;
    for (i,address) in view_iter.enumerate() {
        app_state.borrow_mut().build_ip(address.to_string(), i);
        let ip_address = app_state.view[i];
        app_state.build_ring(ip_address.to_string(), i);
    }
    app_state.ring.sort_by(|a, b| a.hash.cmp(&b.hash));
    let url = format!("{}/kvs/keys/view-change", app_state.random_address());
    let client = reqwest::blocking::Client::new();
    let response = client.put(&url[..])
        .json(&serde_json::json!({
            "view":view_change.view,
            "repl-factor":view_change.repl_factor
        }))
        .send().unwrap()
        .json().unwrap();
    Json(response)
}

#[put("/kvs/<key>", data = "<kvs>")]
pub fn put_kvs(key: &RawStr, kvs:Json<Kvs>, state: State<AppState>) -> Json<PutResult> {
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
pub fn get_kvs(key: &RawStr, state: State<AppState>) -> Json<Kvs> {
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
pub fn get_key_count(state: State<AppState>) -> Json<Kvs> {
    let client = reqwest::blocking::Client::new();
    let url = format!("{}/key_count", state.random_address());
    let response:Kvs = client.get(&url[..])
        .send().unwrap()
        .json().unwrap();
    Json(response)
}

#[get("/kvs/shards")]
pub fn get_shards(state: State<AppState>) -> Json<Kvs> {
    state.print_view();
    let client = reqwest::blocking::Client::new();
    let url = format!("{}/shards", state.random_address());
    let response:Kvs = client.get(&url[..])
        .send().unwrap()
        .json().unwrap();
    Json(response)
}

#[delete("/kvs/<key>")]
pub fn delete_kvs(key: &RawStr, state: State<AppState>) -> Json<Kvs> {
    let client = reqwest::blocking::Client::new();
    let url = format!("{}/kvs/keys/{}", state.choose_address(key), key);
    let response:Kvs = client.delete(&url[..])
        .send().unwrap()
        .json().unwrap();
    println!("response: {:?}", response);
    Json(response)
}
