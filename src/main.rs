#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;
extern crate crypto;
use rocket::http::RawStr;
use serde::{ Serialize, Deserialize };
use rocket_contrib::json::Json;
use rocket::State;
mod app_state;
use app_state::AppState;

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

fn main() {
    let app_state = AppState::default();
    rocket::ignite()
        .manage(app_state)
        .mount("/", routes![index, put_kvs, get_kvs, get_key_count, get_shards])
        .launch();
}
