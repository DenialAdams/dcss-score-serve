#![recursion_limit="128"]

#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

extern crate crawl_model;
extern crate diesel;
extern crate dotenv;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate rocket;
extern crate rocket_contrib;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use rocket_contrib::Template;
use dotenv::dotenv;
use rocket::response::NamedFile;
use rocket::State;
use std::path::{Path, PathBuf};
use std::ops::Deref;

type DatabasePool = r2d2::Pool<r2d2_diesel::ConnectionManager<diesel::SqliteConnection>>;

struct Species(crawl_model::data::Species);

impl Deref for Species {
    type Target = crawl_model::data::Species;
    fn deref(&self) -> &crawl_model::data::Species {
        &self.0
    }
}

impl<'a> rocket::request::FromFormValue<'a> for Species {
    type Error = ();
    fn from_form_value(param: &'a rocket::http::RawStr) -> Result<Species, ()> {
        let species = param.percent_decode_lossy().parse::<crawl_model::data::Species>()?;
        Ok(Species(species))
    }
}

struct Background(crawl_model::data::Background);

impl Deref for Background {
    type Target = crawl_model::data::Background;
    fn deref(&self) -> &crawl_model::data::Background {
        &self.0
    }
}

impl<'a> rocket::request::FromFormValue<'a> for Background {
    type Error = ();
    fn from_form_value(param: &'a rocket::http::RawStr) -> Result<Background, ()> {
        let bg = param.percent_decode_lossy().parse::<crawl_model::data::Background>()?;
        Ok(Background(bg))
    }
}

struct God(crawl_model::data::God);

impl Deref for God {
    type Target = crawl_model::data::God;
    fn deref(&self) -> &crawl_model::data::God {
        &self.0
    }
}

impl<'a> rocket::request::FromFormValue<'a> for God {
    type Error = ();
    fn from_form_value(param: &'a rocket::http::RawStr) -> Result<God, ()> {
        let god = param.percent_decode_lossy().parse::<crawl_model::data::God>()?;
        Ok(God(god))
    }
}

#[derive(FromForm)]
struct GameQuery {
    god: Option<God>,
    background: Option<Background>,
    species: Option<Species>,
    name: Option<String>
}

#[derive(Serialize)]
struct IndexContext {
    games: Vec<FormattedGame>,
}

#[derive(Serialize)]
struct FormattedGame {
    pub real_name: String,
    pub name: String,
    pub score: i64,
    pub species: String,
    pub background: String,
    pub god: String,
    pub runes: i64,
    pub xl: i64,
    pub victory: bool,
}

#[derive(Serialize)]
struct DeathsContext {
    deaths: Vec<FormattedDeath>,
}

#[derive(Serialize)]
struct FormattedDeath {
    pub frequency: i64,
    pub message: String,
}

#[derive(Serialize)]
struct SpeciesContext {
    species: Vec<FormattedSpecies>,
}

#[derive(Serialize)]
struct FormattedSpecies {
    pub frequency: i64,
    pub species: String,
}

impl From<crawl_model::db_model::Game> for FormattedGame {
    fn from(game: crawl_model::db_model::Game) -> FormattedGame {
        let species = unsafe { std::mem::transmute::<i64, crawl_model::data::Species>(game.species_id) };
        let background = unsafe { std::mem::transmute::<i64, crawl_model::data::Background>(game.background_id) };
        let god = unsafe { std::mem::transmute::<i64, crawl_model::data::God>(game.god_id) };
        let real_name = match game.name.as_str() {
            "brick" => "Richard",
            "Peen"|"paul" => "Paul",
            "max"|"PunishedMax" => "Max",
            "daddy"|"fuckboy3000"|"peepeedarts" => "James",
            "sweetBro" => "Luca",
            "hellaJeff" => "Ben H",
            "Richard"|"BoonShekel"|"THEBLIMP" => "Ben S",
            "bobjr93" => "Brennan",
            "jish" => "Josh S",
            "GrapeApe" => "Mason",
            "Doomlord5" => "Dan",
            "MikeyBoy" => "Mike",
            _ => "?"
        };
        let victory = game.is_victory();
        FormattedGame {
            name: game.name,
            real_name: String::from(real_name),
            score: game.score,
            species: format!("{:?}", species),
            background: format!("{:?}", background),
            god: format!("{:?}", god),
            runes: game.runes,
            xl: game.xl,
            victory: victory
        }
    }
}

#[get("/")]
fn hiscores(state: State<DatabasePool>) -> Template {
    let connection = state.get().expect("Timeout waiting for pooled connection");
    let games = {
        use crawl_model::db_schema::games::dsl::*;
        games
            .order(score.desc())
            .limit(100)
            .load::<crawl_model::db_model::Game>(&*connection)
            .expect("Error loading games")
    };
    let formatted_games = games.into_iter().map(|x| x.into() ).collect();
    let context = IndexContext { games: formatted_games };
    Template::render("index", &context)
}

#[get("/?<game_query>")]
fn hi_query(state: State<DatabasePool>, game_query: GameQuery) -> Template {
    let connection = state.get().expect("Timeout waiting for pooled connection");
    let games = {
        use crawl_model::db_schema::games::dsl::*;
        let mut expression = games.into_boxed().order(score.desc()).limit(100);
        if let Some(god) = game_query.god {
            expression = expression.filter(god_id.eq(*god as i64));
        }
        if let Some(background) = game_query.background {
            expression = expression.filter(background_id.eq(*background as i64));
        }
        if let Some(species) = game_query.species {
            expression = expression.filter(species_id.eq(*species as i64));
        }
        if let Some(qname) = game_query.name {
            expression = expression.filter(name.eq(qname))
        }
        expression.load::<crawl_model::db_model::Game>(&*connection).expect("Error loading games")
    };
    let formatted_games = games.into_iter().map(|x| x.into() ).collect();
    let context = IndexContext { games: formatted_games };
    Template::render("index", &context)
}

#[get("/deaths")]
fn deaths(state: State<DatabasePool>) -> Template {
    let connection = state.get().expect("Timeout waiting for pooled connection");
    let deaths: Vec<(String, i64)> = {
        use diesel::dsl::count;
        use diesel::types::BigInt;
        use diesel::dsl::sql;
        use crawl_model::db_schema::games::dsl::*;
        games
            .select((tmsg, sql::<BigInt>("COUNT(games.tmsg)")))
            .order(sql::<BigInt>("COUNT(games.tmsg)").desc())
            .group_by(tmsg)
            .limit(100)
            .load::<_>(&*connection)
            .expect("Error loading games")
    };
    let formatted_deaths = deaths.into_iter().map(|x| FormattedDeath { message: x.0, frequency: x.1 }).collect();
    let context = DeathsContext { deaths: formatted_deaths };
    Template::render("deaths", &context)
}

#[get("/species")]
fn species(state: State<DatabasePool>) -> Template {
    let connection = state.get().expect("Timeout waiting for pooled connection");
    let species: Vec<(i64, i64)> = {
        use diesel::dsl::count;
        use diesel::types::BigInt;
        use diesel::dsl::sql;
        use crawl_model::db_schema::games::dsl::*;
        games
            .select((species_id, sql::<BigInt>("COUNT(games.species_id)")))
            .order(sql::<BigInt>("COUNT(games.species_id)").desc())
            .group_by(species_id)
            .limit(100)
            .load::<_>(&*connection)
            .expect("Error loading games")
    };
    let formatted_species = species.into_iter().map(|x| FormattedSpecies { species: format!("{:?}", unsafe { std::mem::transmute::<i64, crawl_model::data::Species>(x.0) } ), frequency: x.1 }).collect();
    let context = SpeciesContext { species: formatted_species };
    Template::render("species", &context)
}

#[get("/<file..>", rank = 4)]
fn files(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/").join(file)).ok()
}

fn main() {
    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = r2d2_diesel::ConnectionManager::<SqliteConnection>::new(database_url);
    let pool = r2d2::Pool::new(manager).expect("Failed to create pool.");
    rocket::ignite()
        .mount("/", routes![hiscores, files, deaths, hi_query, species])
        .manage(pool)
        .attach(Template::fairing())
        .launch();
}
