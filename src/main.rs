#![feature(proc_macro_hygiene, decl_macro)]

use std::fmt::Debug;
use serde::{Serialize, Deserialize};
use rocket_contrib::templates::Template;
use std::collections::HashMap;
use std::vec::Vec;
use std::string::String;
use std::fs;
use chrono::{Local};


#[macro_use] extern crate rocket;

#[derive(Serialize, Debug)]
struct IndexTemplate {
    time: String,
    user: String,
    materials: HashMap<String, Material>,

}

#[derive(Serialize, Deserialize, Debug)]
struct Material {
    name: String,
    fullname: Option<String>,

}

#[derive(Serialize, Deserialize, Debug)]
struct MaterialDatabase {
    materials: HashMap<String, Material>,
}

#[get("/")]
fn index() -> Template {
    let material_db: MaterialDatabase = match fs::read_to_string("materials.toml") {
        Err(_) => {MaterialDatabase {materials: HashMap::new()}}
        Ok(materials) => {toml::from_str(materials.as_str()).expect("Could not parse meta.toml")}
    };
    let context = IndexTemplate {
        time: Local::now().format("%d.%m.%Y %H:%M").to_string(),
        user: "Annonym".to_string(),
        materials: material_db.materials,
    };
    Template::render("index", context)
}



fn main() {
    rocket::ignite()
        .attach(Template::fairing())
        .mount("/", routes![index])
        .launch();
}
