// main.rs
 
// Jake Armendariz
// Starts rocket server
#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;
extern crate crypto;
mod app_state;
mod routes;
mod api_responses;
mod encryption;
use rocket_contrib::templates::Template;
use crate::routes::{*};
#[cfg(test)] mod tests;

pub fn main() { 
    let app_state = app_state::SharedState::default();
    rocket::ignite()
        .manage(app_state)
        .attach(Template::fairing())
        .mount("/", routes![index, put_kvs, get_kvs, delete_kvs, get_key_count, get_shards, view_change, get_ui, add_address])
        .launch();
}
