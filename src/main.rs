#![recursion_limit="128"]

#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_codegen;
extern crate dotenv;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate rocket;
extern crate rocket_contrib;
extern crate serde;
#[macro_use]
extern crate serde_derive;

mod schema;
mod models;

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use rocket_contrib::Template;
use rocket_contrib::Json;
use dotenv::dotenv;
use rocket::request::Form;
use rocket::http::{Cookie, Cookies, Status};
use rocket::response::NamedFile;
use rocket::State;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use diesel::query_builder::AsChangeset;
use rocket::request::{self, Request, FromRequest};
use rocket::Outcome;

type DatabasePool = r2d2::Pool<r2d2_diesel::ConnectionManager<diesel::SqliteConnection>>;

#[derive(Serialize)]
struct IndexContext {
    morgues: Vec<FormattedMorgue>,
}

#[derive(Serialize)]
struct FormattedMorgue {
    pub real_name: String,
    pub name: String,
    pub score: i64,
    pub race: String,
    pub background: String,
}

impl From<models::DbMorgue> for FormattedMorgue {
    fn from(morgue: models::DbMorgue) -> FormattedMorgue {
        let race = unsafe { std::mem::transmute::<i64, Race>(morgue.race) };
        let background = unsafe { std::mem::transmute::<i64, Background>(morgue.background) };
        let real_name = match morgue.name.as_ref() {
            "brick" => "Richard",
            "Peen" => "Paul",
            "max"|"PunishedMax" => "Max",
            "daddy" => "James",
            "sweetBro" => "Luca",
            "hellaJeff" => "Ben H",
            "Richard" => "Ben S",
            _ => "?"
        };
        FormattedMorgue {
            name: morgue.name,
            real_name: String::from(real_name),
            score: morgue.score,
            race: format!("{:?}", race),
            background: format!("{:?}", background)
        }
    }
}


#[get("/")]
fn hiscores(state: State<DatabasePool>) -> Template {
    let connection = state.get().expect("Timeout waiting for pooled connection");
    let mut morgues = {
        use schema::morgues::dsl::*;
        morgues
            .order(score.desc())
            .limit(100)
            .load::<models::DbMorgue>(&*connection)
            .expect("Error loading morgues")
    };
    let formatted_morgues = morgues.into_iter().map(|x| x.into() ).collect();
    let context = IndexContext { morgues: formatted_morgues };
    Template::render("index", &context)
}

#[get("/<file..>", rank = 2)]
fn files(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/").join(file)).ok()
}

fn main() {
    dotenv().ok();

    let config = r2d2::Config::default();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = r2d2_diesel::ConnectionManager::<SqliteConnection>::new(database_url);
    let pool = r2d2::Pool::new(config, manager).expect("Failed to create pool.");
    rocket::ignite()
        .mount("/", routes![hiscores, files])
        .manage(pool)
        .attach(Template::fairing())
        .launch();
}

#[derive(Debug, Clone, Copy)]
#[repr(i64)]
enum Race {
    Barachi = 0,
    Centaur,
    DeepDwarf,
    DeepElf,
    Demigod,
    Demonspawn,
    Draconian,
    RedDraconian,
    WhiteDraconian,
    GreenDraconian,
    YellowDraconian,
    GreyDraconian,
    BlackDraconian,
    PurpleDraconian,
    MottledDraconian,
    PaleDraconian,
    Felid,
    Formicid,
    Gargoyle,
    Ghoul,
    Gnoll,
    Halfling,
    HighElf,
    HillOrc,
    Human,
    Kobold,
    Merfolk,
    Minotaur,
    Mummy,
    Naga,
    Ocotopode,
    Ogre,
    Spriggan,
    Tengu,
    Troll,
    Vampire,
    VineStalker,
}

#[derive(Debug, Clone, Copy)]
#[repr(i64)]
enum Background {
    Fighter = 0,
    Gladiator,
    Monk,
    Hunter,
    Assassin,
    Berserker,
    AbyssalKnight,
    ChaosKnight,
    Skald,
    Enchanter,
    Transmuter,
    ArcaneMarksman,
    Warper,
    Wizard,
    Conjurer,
    Summoner,
    Necromancer,
    FireElementalist,
    IceElementalist,
    AirElementalist,
    EarthElementalist,
    VenomMage,
    Artificer,
    Wanderer,
}
