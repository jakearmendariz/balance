// api_responses.rs

// Jake Armendariz
// structures for the responses from kvs

use serde::{ Serialize, Deserialize };

#[derive(Deserialize, Debug, Serialize)]
pub struct GetSuccess {
    pub value: String
}

#[derive(Deserialize, Debug, Serialize)]
pub struct GetError {
    pub error: String
}

#[derive(Deserialize, Debug, Serialize)]
pub enum GetResult {
    Successful(GetSuccess),
    Unsuccessful(GetError)
}

#[derive(Deserialize, Debug, Serialize)]
pub struct KeyCount {
    pub key_count: u8
}

#[derive(Deserialize, Debug, Serialize)]
pub struct PutResult {
    pub message: String,
    replaced: bool
}

#[derive(Deserialize, Debug, Serialize)]
pub struct DeleteResult {
    message: String,
    does_exist:bool 
}

#[derive(Deserialize, Debug, Serialize)]
pub struct ViewChange {
    pub view: String,
    pub repl_factor: u8
}

#[derive(Deserialize, Debug, Serialize, Default)]
pub struct ViewChangeResult {
    message: String,
    shards: Vec<String>
}

#[derive(Deserialize, Debug, Serialize, Default)]
pub struct BaseResponse {
    pub successful:bool,
    pub message:String,
}
