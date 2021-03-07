use rocket_contrib::templates::Template;
use crate::routes::{*};
use crate::app_state::SharedState;

pub fn rocket() -> rocket::Rocket { 
    let app_state = SharedState::default();
    rocket::ignite()
        .manage(app_state)
        .attach(Template::fairing())
        .mount("/", routes![index, put_kvs, get_kvs, delete_kvs, get_key_count, get_shards, view_change, get_ui, add_address])
        
}

#[cfg(test)]
mod test {
    use super::rocket;
    use rocket::local::Client;
    use rocket::http::Status;

    #[test]
    fn hello_world() {
        let client = Client::new(rocket()).expect("valid rocket instance");
        let mut response = client.get("/kvs/keys/dipshit").dispatch();
        assert_eq!(response.status(), Status::Ok);
    }
}