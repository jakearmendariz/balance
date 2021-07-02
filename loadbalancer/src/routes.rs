// routes.rs
 
// Jake Armendariz
// Routes for forwarding requests to distributed kvs
extern crate dotenv_codegen;
extern crate crypto;
extern crate reqwest;
extern crate lazy_static;

extern crate dotenv;

use dotenv::dotenv;

use rocket::State;
use rocket::http::RawStr;
use rocket_contrib::json::Json;
use crate::app_state::{SharedState, AppState, KvsError, FORWARDING_ERROR, JSON_DECODING_ERROR};
use crate::api_responses::{*};
use crate::encryption::{*};
use rocket_contrib::templates::Template;


lazy_static::lazy_static! {
    static ref CLIENT:reqwest::blocking::Client = reqwest::blocking::Client::new();
}

#[get("/")]
pub fn index() -> &'static str {
    "Distributed Key Value Store"
}

// Append to list of views
#[post("/kvs/view-change", data = "<view_change>")]
pub fn add_address(view_change:Json<ViewChange>, shared_state: State<SharedState>) -> Result<Json<BaseResponse>, KvsError> {
    let mut app_state = shared_state.state.write().expect("lock shared data");
    if app_state.view.iter().any(|&i| i.to_string() == view_change.view) {
        return Ok(Json(BaseResponse {message: "View change successful".to_string(), successful: true}))
    }
    let i = app_state.length;
    let address = &view_change.view;
    app_state.build_ip(address.to_string(), i);
    app_state.ring.sort_by(|a, b| a.hash.cmp(&b.hash));
    app_state.repl_factor = view_change.repl_factor;
    let url = format!("{}/kvs/view-change", app_state.random_address());
    let body = &serde_json::json!({
        "view":view_change.view,
        "repl-factor":view_change.repl_factor,
        "access_token":dotenv!("ACCESS_TOKEN")
    });
    let response = match CLIENT.put(&url[..])
        .json(body).send() {
            Ok(_) => Ok(Json(BaseResponse {message: "View change successful".to_string(), successful: true})),
            Err(_) => Err(KvsError(FORWARDING_ERROR.to_string()))
        };
    response
}

#[put("/kvs/view-change", data = "<view_change>")]
pub fn view_change(view_change:Json<ViewChange>, shared_state: State<SharedState>) -> Result<Json<ViewChangeResult>, KvsError> {
    let mut app_state = shared_state.state.write().expect("lock shared data");
    app_state.repl_factor = view_change.repl_factor;
    let view_iter = view_change.view.split(",");
    app_state.length = 0;

    // Build the hash ring for every address in the view.
    for (i,address) in view_iter.enumerate() {
        app_state.build_ip(address.to_string(), i);
        let ip_address = app_state.view[i];
        app_state.build_ring(ip_address.to_string(), i);
    }
    app_state.ring.sort_by(|a, b| a.hash.cmp(&b.hash));

    // Send the view change
    let url = format!("{}/kvs/view-change", app_state.random_address());
    let body = &serde_json::json!({
        "view":view_change.view,
        "repl-factor":view_change.repl_factor,
        "access_token":dotenv!("ACCESS_TOKEN")
    });
    let response = match CLIENT.put(&url[..])
        .json(body).send() {
            Ok(response) => Ok(Json(response.json().unwrap())),
            Err(_) => Err(KvsError(FORWARDING_ERROR.to_string()))
        };
    response
}

// builds the body for a put request
// if encrypted global variable then encrypt
fn put_body(state:AppState, kvs:Json<GetSuccess>) -> serde_json::Value {
    if state.encrypt {
        serde_json::json!({
            "value":encrypt(kvs.value.to_string()),
            "access_token":dotenv!("ACCESS_TOKEN")
        })
    } else {
        serde_json::json!({
            "value":kvs.value,
            "access_token":dotenv!("ACCESS_TOKEN")
        })
    }
}

fn put_response(response:reqwest::blocking::Response ) -> Result<Json<PutResult>, KvsError> {
    if response.status().is_success() {
        match response.json::<PutSuccess>() {
            Ok(response) => {
                Ok(Json(PutResult::Successful(response)))
            },
            Err(e) => {
                eprintln!("successful request, unable to decode: {:?}", e);
                Err(KvsError(JSON_DECODING_ERROR.to_string()))
            },
        }
    }else {
        match response.json::<Error>() {
            Ok(response_json) => Ok(Json(PutResult::Unsuccessful(response_json))),
            Err(e) => {
                eprintln!("unsuccessful request, unable to decode: {:?}", e);
                Err(KvsError(JSON_DECODING_ERROR.to_string()))
            },
        }
    }
}
#[put("/kvs/keys/<key>", data = "<kvs>")]
pub fn put_kvs(key: &RawStr, kvs:Json<GetSuccess>, shared_state: State<SharedState>) -> Result<Json<PutResult>, KvsError> {
    let state = shared_state.state.read().expect("lock shared data");
    let url = format!("{}/kvs/keys/{}", state.choose_address(key)?, key);
    let body = put_body(*state, kvs);
    let response = match CLIENT.put(&url[..])
        .json(&body).send() {
            Ok(response) => put_response(response),
            Err(_) => Err(KvsError(FORWARDING_ERROR.to_string()))
        };
    response
}

fn read_response(state:AppState, response:reqwest::blocking::Response ) -> Result<Json<GetResult>, KvsError> {
    if response.status().is_success() {
        match response.json::<GetSuccess>() {
            Ok(mut response) => {
                if state.encrypt {
                    response.value = decrypt(response.value);
                }
                Ok(Json(GetResult::Successful(response)))
            },
            Err(e) => {
                eprintln!("successful request, unable to decode: {:?}", e);
                Err(KvsError(JSON_DECODING_ERROR.to_string()))
            },
        }
    }else {
        match response.json::<GetError>() {
            Ok(response_json) => Ok(Json(GetResult::Unsuccessful(response_json))),
            Err(e) => {
                eprintln!("unsuccessful request, unable to decode: {:?}", e);
                Err(KvsError(JSON_DECODING_ERROR.to_string()))
            },
        }
    }
    
}

#[get("/kvs/keys/<key>")]
pub fn get_kvs(key: &RawStr, shared_state: State<SharedState>) -> Result<Json<GetResult>, KvsError> {
    let state = shared_state.state.read().expect("lock shared data");
    let url = format!("{}/kvs/keys/{}", state.choose_address(key)?, key);
    println!("get_kvs()\n");
    let body = &serde_json::json!({
        "access_token":dotenv!("ACCESS_TOKEN")
    });
    println!("access token found, {}\n", url);
    match CLIENT.get(&url[..]).json(body).send() {
        Ok(response) => read_response(*state, response),
        Err(_) => Err(KvsError(FORWARDING_ERROR.to_string()))
    }
}

#[get("/kvs/ui/<key>")]
pub fn get_ui(key: &RawStr, shared_state: State<SharedState>) -> Template {
    match get_kvs(key, shared_state) {
        Ok(response) => {
            let value = match response.into_inner() {
                GetResult::Successful(kvs)  => kvs.value,
                GetResult::Unsuccessful(error) => error.error
            };
            let mut context = std::collections::HashMap::new();
            context.insert("value", value);
            Template::render("index", &context)
        },
        Err(e) => Template::render("error", &e)
    }
}


#[get("/kvs/key-count")]
pub fn get_key_count(shared_state: State<SharedState>) -> Result<Json<KeyCount>, KvsError> {
    let state = shared_state.state.read().expect("lock shared data");
    let url = format!("{}/kvs/key-count", state.random_address());
    let response = match CLIENT.get(&url[..]).send() {
            Ok(response) => Ok(Json(response.json().unwrap())),
            Err(_) => Err(KvsError(FORWARDING_ERROR.to_string()))
        };
    response
}

#[get("/kvs/shards")]
pub fn get_shards(shared_state: State<SharedState>) -> Result<Json<KeyCount>, KvsError> {
    let state = shared_state.state.read().expect("lock shared data");
    let url = format!("{}/kvs/shards", state.random_address());
    let response = match CLIENT.get(&url[..]).send() {
        Ok(response) => Ok(Json(response.json().unwrap())),
        Err(_) => Err(KvsError(FORWARDING_ERROR.to_string()))
    };
    response
}

#[delete("/kvs/keys/<key>")]
pub fn delete_kvs(key: &RawStr, shared_state: State<SharedState>) -> Result<Json<DeleteResult>, KvsError> {
    let state = shared_state.state.read().expect("lock shared data");
    let url = format!("{}/kvs/keys/{}", state.choose_address(key)?, key);
    let response = match CLIENT.delete(&url[..])
        .send() {
            Ok(response) => Ok(Json(response.json().unwrap())),
            Err(_) => Err(KvsError(FORWARDING_ERROR.to_string()))
        };
    response
}
