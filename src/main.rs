#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;

use rocket::http::RawStr;
use serde::{ Serialize, Deserialize };
use rocket_contrib::json::Json;
use rand::Rng;
use rocket::State;

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
    println!("view:{:?}", state.view);
    let client = reqwest::blocking::Client::new();
    let url = format!("{}/kvs/keys/{}", state.choose_address(key), key);
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
    println!("view:{:?}", state.view);
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
pub struct AppState<'a> {
    pub repl_factor:u8,
    pub view: &'a str
}

impl Default for AppState<'static> {
    fn default() -> Self {
        let view_str = std::env::var("VIEW").unwrap();
        // let view_iter = view_str.split(",");
        // let mut view:Vec<String> = Vec::new();
        // for vstr in view_iter {
        //     view.push(vstr.to_string());
        // }
        return AppState<'static> {
            repl_factor: std::env::var("REPL_FACTOR").unwrap()
                            .parse::<u8>().unwrap(),
            view: &view_str[..]
        };
    }
}

impl AppState<'static> {
    fn get_view(self) -> Vec<String> {
        let view_iter = self.view.to_string().split(",");
        let mut view:Vec<String> = Vec::new();
        for vstr in view_iter {
            view.push(vstr.to_string());
        }
        return view;
    }

    fn choose_address(self, _key:&RawStr) -> String {
        let mut rng = rand::thread_rng();
        let view = self.get_view();
        let i:usize = rng.gen_range(0..view.len());
        return format!("http://localhost:{}", view[i])
    }

    fn random_address(self) -> String {
        let mut rng = rand::thread_rng();
        let view = self.get_view();
        let i:usize = rng.gen_range(0..view.len());
        return format!("http://localhost:{}", view[i])
    }
}

// fn choose_address(self, _key:&RawStr) -> String {
//     let mut rng = rand::thread_rng();
//     let i:usize = rng.gen_range(0..self.view.len());
//     return format!("http://localhost:{}", self.view[i])
// }

// fn random_address(view:) -> String {
//     let mut rng = rand::thread_rng();
//     let i:usize = rng.gen_range(0..self.view.len());
//     return format!("http://localhost:{}", self.view[i])
// }

fn main() {
    let app_state = AppState::default();
    rocket::ignite()
        .manage(app_state)
        .mount("/", routes![index, put_kvs, get_kvs, get_key_count, get_shards])
        .launch();
}
