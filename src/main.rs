#![feature(proc_macro_hygiene, decl_macro)]

use std::{cell::RefCell, fmt::Debug, sync::{Mutex, atomic::AtomicBool}};
use serde::{Serialize, Deserialize};
use rocket_contrib::templates::Template;
use rocket::request::Form;
use rocket::response::Redirect;
use rocket::State;
use std::collections::HashMap;
use std::vec::Vec;
use std::string::String;
use std::fs;
use chrono::{NaiveDateTime, Local};
use std::io::prelude::*;
use slug::slugify;




#[macro_use] extern crate rocket

;
struct LoginState {
    /// Aktuell angemeldeter Nutzer
    user: Mutex<RefCell<String>>,
    /// Flag für Loginstatus
    unlocked: AtomicBool,
}


#[derive(Serialize, Debug)]
struct EmptyTemplate {
}

#[derive(Serialize, Debug)]
struct EntryTemplate {
    time: String,
    timestamp: i64,
    user: String,
    materials: HashMap<String, Material>,
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
fn new_entry(login_state: State<LoginState>) -> Template {
    let material_db: MaterialDatabase = match fs::read_to_string("materials.toml") {
        Err(_) => {MaterialDatabase {materials: HashMap::new()}}
        Ok(materials) => {toml::from_str(materials.as_str()).expect("Could not parse meta.toml")}
    };
    let now = Local::now();
    let context = EntryTemplate {
        time: now.format("%d.%m.%Y %H:%M").to_string(),
        timestamp: now.timestamp(),
        user: login_state.user.lock().unwrap().borrow().clone(),
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
    let slug = slugify(&format!("{}_{}", input.user, input.timestamp));
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


#[derive(Serialize, Debug)]
struct MaterialTemplate {
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
    let slug = slugify(&input.name);
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

/**
Parameter für das Login API und eventuell später eine HTML Form
 */
#[derive(Serialize, FromForm, Debug)]
struct LoginForm {
    /// Name des Nutzers
    username: String,
    /// Auth Token für API Clients (NYI)
    token: Option<String>,
}

#[post("/unlock", data = "<input>")]
fn post_login(login_state: State<LoginState> ,input: Form<LoginForm>) -> Redirect {
    login_state.user.lock().unwrap().replace(input.username.clone());
    login_state.unlocked.store(true, std::sync::atomic::Ordering::Relaxed);
    Redirect::to("/entry/new")
}

#[post("/lock")]
fn post_logout(login_state: State<LoginState>) -> Redirect {
    login_state.user.lock().unwrap().replace(String::new());
    login_state.unlocked.store(false, std::sync::atomic::Ordering::Relaxed);
    Redirect::to("/")
}

/**
Die Landingpage soll automatisch auf die Eingabemaske umschalten sobalt ein Nutzer Autentifiziert ist,
dafür wird per XHR 1/s der '/login' enpoint abgefragt.
*/
#[get("/")]
fn get_login(login_state: State<LoginState>) -> String {
    return login_state.unlocked.load(std::sync::atomic::Ordering::Relaxed).to_string();
}

#[get("/")]
fn landing() -> Template {
    Template::render("landing", EmptyTemplate {})
}

fn main() {
    let login_state = LoginState {
        user: Mutex::new(RefCell::new(String::new())),
        unlocked: AtomicBool::new(false),
    };
    rocket::ignite()
        .attach(Template::fairing())
        .mount("/login", routes![post_login, post_logout, get_login])
        .mount("/mat", routes![new_mat, post_mat])
        .mount("/entry", routes![new_entry, post_entry])
        .mount("/", routes![landing])
        .manage(login_state)
        .launch();
}
