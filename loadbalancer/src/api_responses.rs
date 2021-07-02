// api_responses.rs

// Jake Armendariz
// structures for the responses from kvs

use serde::{ Serialize, Deserialize };
use std::fmt;


#[derive(Deserialize, Debug, Serialize)]
pub struct GetSuccess {
    pub value: String
}

// #[derive(Debug, Responder)]
// enum Error {
//     #[response(status = 400)]
//     BadRequest('static str),
//     #[response(status = 404)]
//     NotFound('static str),
//     #[response(status = 400)]
//     InternalServer('static str)
// }

#[derive(Deserialize, Debug, Serialize)]
pub struct GetError {
    pub error: String
}

#[derive(Deserialize, Debug, Serialize)]
pub enum GetResult {
    Successful(GetSuccess),
    Unsuccessful(GetError),
}

#[derive(Deserialize, Debug, Serialize)]
pub struct KeyCount {
    pub key_count: u8
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
      write!(f, "error:{}, message: {}", self.error, self.message)
    }
}

#[derive(Deserialize, Debug, Serialize)]
pub struct Error {
    pub error: String,
    pub message: String
}

#[derive(Deserialize, Debug, Serialize)]
pub struct PutSuccess {
    pub message: String,
    replaced: bool
}

#[derive(Deserialize, Debug, Serialize)]
pub enum PutResult {
    Successful(PutSuccess),
    Unsuccessful(Error),
    String(String)
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
