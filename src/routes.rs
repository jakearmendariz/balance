extern crate crypto;
extern crate reqwest;

use rocket::http::RawStr;
use serde::{ Serialize, Deserialize };

use rocket_contrib::json::Json;
use rocket::State;
use crate::app_state::{SharedState, KvsError, FORWARDING_ERROR, JSON_DECODING_ERROR};
use std::borrow::BorrowMut;

#[get("/")]
pub fn index() -> &'static str {
    "Distributed Key Value Store"
}

#[derive(Deserialize, Debug, Serialize)]
pub struct GetSuccess {
    value: String
}

#[derive(Deserialize, Debug, Serialize)]
pub struct GetError {
    error: String
}

#[derive(Deserialize, Debug, Serialize)]
pub enum GetResult {
    Successful(GetSuccess),
    Unsuccessful(GetError)
}

#[derive(Deserialize, Debug, Serialize)]
pub struct KeyCount {
    key_count: u8
}

#[derive(Deserialize, Debug, Serialize)]
pub struct PutResult {
    message: String,
    replaced: bool
}

#[derive(Deserialize, Debug, Serialize)]
pub struct DeleteResult {
    message: String,
    does_exist:bool 
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

#[put("/kvs/view-change", data = "<view_change>")]
pub fn view_change(view_change:Json<ViewChange>, shared_state: State<SharedState>) -> Result<Json<ViewChangeResult>, KvsError> {
    let mut app_state = shared_state.state.write().expect("lock shared data");
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
    let client = reqwest::blocking::Client::new();
    let response = match client.put(&url[..])
        .json(&serde_json::json!({
            "view":view_change.view,
            "repl-factor":view_change.repl_factor
        })).send() {
            Ok(response) => Ok(Json(response.json().unwrap())),
            Err(_) => Err(KvsError(FORWARDING_ERROR.to_string()))
        };
    response
}

#[put("/kvs/keys/<key>", data = "<kvs>")]
pub fn put_kvs(key: &RawStr, kvs:Json<GetSuccess>, shared_state: State<SharedState>) -> Result<Json<PutResult>, KvsError> {
    let state = shared_state.state.read().expect("lock shared data");
    let client = reqwest::blocking::Client::new();
    let url = format!("{}/kvs/keys/{}", state.choose_address(key)?, key);
    let value = &kvs.value;
    let response = match client.put(&url[..])
        .json(&serde_json::json!({
            "value":state.encrypt(value.to_string())
        })).send() {
            Ok(response) => Ok(Json(response.json().unwrap())),
            Err(_) => Err(KvsError(FORWARDING_ERROR.to_string()))
        };
    response
}

#[get("/kvs/keys/<key>")]
pub fn get_kvs(key: &RawStr, shared_state: State<SharedState>) -> Result<Json<GetResult>, KvsError> {
    let state = shared_state.state.read().expect("lock shared data");
    let client = reqwest::blocking::Client::new();
    let url = format!("{}/kvs/keys/{}", state.choose_address(key)?, key);
    println!("url: {}", url);
    let response = match client.get(&url[..]).send() {
        Ok(response) => {
            if response.status().is_success() {
                match response.json() {
                    Ok(response_json) => {
                        let mut res:GetSuccess = response_json;
                        res.value = state.decrypt(res.value);
                        return Ok(Json(GetResult::Successful(res)));
                    },
                    Err(e) => {
                        eprintln!("successful request, unable to decode: {:?}", e);
                    },
                }
            }else {
                match response.json() {
                    Ok(response_json) => return Ok(Json(GetResult::Unsuccessful(response_json))),
                    Err(e) => {
                        eprintln!("unsuccessful request, unable to decode: {:?}", e);
                    },
                }
            }
            Err(KvsError(JSON_DECODING_ERROR.to_string()))
        },
        Err(_) => {
            Err(KvsError(FORWARDING_ERROR.to_string()))
        }
    };
    response
}

#[get("/kvs/key-count")]
pub fn get_key_count(shared_state: State<SharedState>) -> Result<Json<KeyCount>, KvsError> {
    let state = shared_state.state.read().expect("lock shared data");
    let client = reqwest::blocking::Client::new();
    let url = format!("{}/kvs/key-count", state.random_address());
    println!("url: {}", url);
    let response = match client.get(&url[..])
        .send() {
            Ok(response) => Ok(Json(response.json().unwrap())),
            Err(e) => {
                println!("error:{}", e);
                Err(KvsError(FORWARDING_ERROR.to_string()))
            }
        };
    response
}

#[get("/kvs/shards")]
pub fn get_shards(shared_state: State<SharedState>) -> Result<Json<KeyCount>, KvsError> {
    let state = shared_state.state.read().expect("lock shared data");
    let client = reqwest::blocking::Client::new();
    let url = format!("{}/kvs/shards", state.random_address());
    let response = match client.get(&url[..])
    .send() {
        Ok(response) => Ok(Json(response.json().unwrap())),
        Err(_) => Err(KvsError(FORWARDING_ERROR.to_string()))
    };
    response
}

#[delete("/kvs/keys/<key>")]
pub fn delete_kvs(key: &RawStr, shared_state: State<SharedState>) -> Result<Json<DeleteResult>, KvsError> {
    let state = shared_state.state.read().expect("lock shared data");
    let client = reqwest::blocking::Client::new();
    let url = format!("{}/kvs/keys/{}", state.choose_address(key)?, key);
    let response = match client.delete(&url[..])
        .send() {
            Ok(response) => Ok(Json(response.json().unwrap())),
            Err(_) => Err(KvsError(FORWARDING_ERROR.to_string()))
        };
    response
}
