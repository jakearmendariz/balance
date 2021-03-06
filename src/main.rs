#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;
extern crate crypto;
mod app_state;
mod routes;


fn main() { 
    let app_state = app_state::SharedState::default();
    rocket::ignite()
        .manage(app_state)
        .mount("/", routes![routes::index, routes::put_kvs, routes::get_kvs, routes::delete_kvs, routes::get_key_count, routes::get_shards, routes::view_change])
        .launch();
}
