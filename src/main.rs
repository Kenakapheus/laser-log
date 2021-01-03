#![feature(proc_macro_hygiene, decl_macro)]

use std::fmt::Debug;
use serde::{Serialize, Deserialize};
use rocket::response::{NamedFile};
use rocket::response::status::NotFound;
use rocket_contrib::templates::Template;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::vec::Vec;
use std::string::String;
use std::fs;
use chrono::{DateTime, Local};


#[macro_use] extern crate rocket;

#[derive(Serialize, Debug)]
struct IndexTemplate {
    time: String,
    user: String,
}

#[get("/")]
fn index() -> Template {
    let context = IndexTemplate {
        time: Local::now().format("%d.%m.%Y %H:%M").to_string(),
        user: "Annonym".to_string(),
    };
    Template::render("index", context)
}



fn main() {
    rocket::ignite()
        .attach(Template::fairing())
        .mount("/", routes![index])
        .launch();
}
