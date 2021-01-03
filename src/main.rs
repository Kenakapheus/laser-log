#![feature(proc_macro_hygiene, decl_macro)]

use std::fmt::Debug;
use serde::{Serialize, Deserialize};
use rocket_contrib::templates::Template;
use rocket::request::Form;
use rocket::response::Redirect;
use std::collections::HashMap;
use std::vec::Vec;
use std::string::String;
use std::fs;
use chrono::{NaiveDateTime, Local};
use regex::Regex;
use std::io::prelude::*;


#[macro_use] extern crate rocket;
#[macro_use] extern crate lazy_static;

lazy_static! {
    static ref SLUG_RE: Regex = Regex::new(r"([a-z0-9_]+)").unwrap();
}

#[derive(Serialize, Debug)]
struct IndexTemplate {
    time: String,
    timestamp: i64,
    user: String,
    materials: HashMap<String, Material>,
}

#[derive(Serialize, Debug)]
struct MaterialTemplate {
}

#[derive(Serialize, Deserialize, Debug)]
struct Material {
    name: String,
    fullname: Option<String>,

}

#[derive(Serialize, Deserialize, Debug)]
struct LogMaterial {
    slug: String,
    name: String,
    fullname: Option<String>,

}

#[derive(Serialize, Deserialize, Debug)]
struct MaterialDatabase {
    materials: HashMap<String, Material>,
}

#[derive(Serialize, FromForm, Debug)]
struct MaterialForm {
    name: String,
    fullname: String,
}

#[derive(Serialize, Debug)]
struct Entry {
    timestamp: String,
    user: String,
    thickness: f32,
    comment: String,
    material: LogMaterial,
}

#[derive(Serialize, FromForm, Debug)]
struct EntryForm {
    timestamp: i64,
    user: String,
    material: String,
    thickness: f32,
    comment: String,
}

#[get("/new")]
fn new_entry() -> Template {
    let material_db: MaterialDatabase = match fs::read_to_string("materials.toml") {
        Err(_) => {MaterialDatabase {materials: HashMap::new()}}
        Ok(materials) => {toml::from_str(materials.as_str()).expect("Could not parse meta.toml")}
    };
    let now = Local::now();
    let context = IndexTemplate {
        time: now.format("%d.%m.%Y %H:%M").to_string(),
        timestamp: now.timestamp(),
        user: "Annonym".to_string(),
        materials: material_db.materials,
    };
    Template::render("new-entry", context)
}

#[post("/new", data = "<input>")]
fn post_entry(input: Form<EntryForm>) -> Redirect {
    let material_db: MaterialDatabase = match fs::read_to_string("materials.toml") {
        Err(_) => {MaterialDatabase {materials: HashMap::new()}}
        Ok(materials) => {toml::from_str(materials.as_str()).expect("Could not parse materials.toml")}
    };
    let mat = material_db.materials.get(&input.material).unwrap();
    let material = LogMaterial {
        name: mat.name.clone(),
        fullname: mat.fullname.clone(),
        slug: input.material.clone(),
    };
    let entry = Entry {
        timestamp: NaiveDateTime::from_timestamp(input.timestamp, 0).to_string(),
        user: input.user.clone(),
        material,
        thickness: input.thickness,
        comment: input.comment.clone(),
    };
    let slug = to_slug(&format!("{}_{}", input.user, input.timestamp));
    let mut entrys = HashMap::new();
    entrys.insert(slug, entry);
    match toml::to_string(&entrys) {
        Ok(s) => {
            match fs::OpenOptions::new().write(true).append(true).create(true).open("log.toml") {
                Ok(mut f) => {writeln!(f, "\n{}\n", s).expect("Error while writing Log file");},
                Err(e) => {print!("Error while opening Log file {:?}\n", e);}
            };
        },
        Err(e) => {print!("Error while Serializing Log {:?}\n", e)}
    };
    Redirect::to("/entry/new")
}

#[get("/new")]
fn new_mat() -> Template {
    let context = MaterialTemplate {};
    Template::render("new-material", context)
}

#[post("/new", data = "<input>")]
fn post_mat(input: Form<MaterialForm>) -> Redirect {
    let material = Material{
        name: input.name.clone(),
        fullname: match input.fullname.as_str() {"" => None, s => Some(s.to_string())},
    };
    let slug = to_slug(&input.name);
    let mut materials = HashMap::new();
    materials.insert(slug, material);
    match toml::to_string(&MaterialDatabase{materials}) {
        Ok(s) => {
            match fs::OpenOptions::new().write(true).append(true).open("materials.toml") {
                Ok(mut f) => {writeln!(f, "\n{}\n", s).expect("Error while writing Material file");},
                Err(e) => {print!("Error while opening Material file {:?}\n", e);}
            };
        },
        Err(e) => {print!("Error while Serializing Material {:?}\n", e)}
    };
    Redirect::to("/entry/new")
}

fn to_slug(input: &String) -> String {
            let mut slug = String::new();
            for cap in SLUG_RE.captures_iter(input.to_lowercase().as_str()) {
                slug.push_str(&cap[1]);
            }
            slug
}

fn main() {
    rocket::ignite()
        .attach(Template::fairing())
        .mount("/mat", routes![new_mat, post_mat])
        .mount("/entry", routes![new_entry, post_entry])
        .launch();
}
