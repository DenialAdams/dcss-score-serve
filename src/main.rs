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

impl<'a> rocket::request::FromParam<'a> for Species {
    type Error = ();
    fn from_param(param: &'a rocket::http::RawStr) -> Result<Species, ()> {
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

impl<'a> rocket::request::FromParam<'a> for Background {
    type Error = ();
    fn from_param(param: &'a rocket::http::RawStr) -> Result<Background, ()> {
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

impl<'a> rocket::request::FromParam<'a> for God {
    type Error = ();
    fn from_param(param: &'a rocket::http::RawStr) -> Result<God, ()> {
        let god = param.percent_decode_lossy().parse::<crawl_model::data::God>()?;
        Ok(God(god))
    }
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
    pub god: String
}

impl From<crawl_model::db_model::Game> for FormattedGame {
    fn from(game: crawl_model::db_model::Game) -> FormattedGame {
        let species = unsafe { std::mem::transmute::<i64, crawl_model::data::Species>(game.species_id) };
        let background = unsafe { std::mem::transmute::<i64, crawl_model::data::Background>(game.background_id) };
        let god = unsafe { std::mem::transmute::<i64, crawl_model::data::God>(game.god_id) };
        let real_name = match game.name.as_str() {
            "brick" => "Richard",
            "Peen" => "Paul",
            "max"|"PunishedMax" => "Max",
            "daddy"|"fuckboy3000" => "James",
            "sweetBro" => "Luca",
            "hellaJeff" => "Ben H",
            "Richard"|"BoonShekel" => "Ben S",
            _ => "?"
        };
        FormattedGame {
            name: game.name,
            real_name: String::from(real_name),
            score: game.score,
            species: format!("{:?}", species),
            background: format!("{:?}", background),
            god: format!("{:?}", god),
        }
    }
}

#[get("/<species>", rank = 1)]
fn best_species(state: State<DatabasePool>, species: Species) -> Template {
    let connection = state.get().expect("Timeout waiting for pooled connection");
    let games = {
        use crawl_model::db_schema::games::dsl::*;
        games
            .filter(species_id.eq(*species as i64))
            .order(score.desc())
            .limit(100)
            .load::<crawl_model::db_model::Game>(&*connection)
            .expect("Error loading games")
    };
    let formatted_games = games.into_iter().map(|x| x.into() ).collect();
    let context = IndexContext { games: formatted_games };
    Template::render("index", &context)
}

#[get("/<bg>", rank = 2)]
fn best_bg(state: State<DatabasePool>, bg: Background) -> Template {
    let connection = state.get().expect("Timeout waiting for pooled connection");
    let games = {
        use crawl_model::db_schema::games::dsl::*;
        games
            .filter(background_id.eq(*bg as i64))
            .order(score.desc())
            .limit(100)
            .load::<crawl_model::db_model::Game>(&*connection)
            .expect("Error loading games")
    };
    let formatted_games = games.into_iter().map(|x| x.into() ).collect();
    let context = IndexContext { games: formatted_games };
    Template::render("index", &context)
}

#[get("/<god>", rank = 3)]
fn best_god(state: State<DatabasePool>, god: God) -> Template {
    let connection = state.get().expect("Timeout waiting for pooled connection");
    let games = {
        use crawl_model::db_schema::games::dsl::*;
        games
            .filter(god_id.eq(*god as i64))
            .order(score.desc())
            .limit(100)
            .load::<crawl_model::db_model::Game>(&*connection)
            .expect("Error loading games")
    };
    let formatted_games = games.into_iter().map(|x| x.into() ).collect();
    let context = IndexContext { games: formatted_games };
    Template::render("index", &context)
}

#[get("/u/<player>")]
fn best_player(state: State<DatabasePool>, player: String) -> Template {
    let connection = state.get().expect("Timeout waiting for pooled connection");
    let games = {
        use crawl_model::db_schema::games::dsl::*;
        games
            .filter(name.eq(player))
            .order(score.desc())
            .limit(100)
            .load::<crawl_model::db_model::Game>(&*connection)
            .expect("Error loading games")
    };
    let formatted_games = games.into_iter().map(|x| x.into() ).collect();
    let context = IndexContext { games: formatted_games };
    Template::render("index", &context)
}

#[get("/<species>/<bg>")]
fn best_combo(state: State<DatabasePool>, species: Species, bg: Background) -> Template {
    let connection = state.get().expect("Timeout waiting for pooled connection");
    let games = {
        use crawl_model::db_schema::games::dsl::*;
        games
            .filter(species_id.eq(*species as i64))
            .filter(background_id.eq(*bg as i64))
            .order(score.desc())
            .limit(100)
            .load::<crawl_model::db_model::Game>(&*connection)
            .expect("Error loading games")
    };
    let formatted_games = games.into_iter().map(|x| x.into() ).collect();
    let context = IndexContext { games: formatted_games };
    Template::render("index", &context)
}

#[get("/<bg>/<species>", rank = 2)]
fn best_combo_inverted(state: State<DatabasePool>, species: Species, bg: Background) -> Template {
    best_combo(state, species, bg)
}

#[get("/<species>/<bg>/<god>")]
fn best_combo_and_god(state: State<DatabasePool>, species: Species, bg: Background, god: God) -> Template {
    let connection = state.get().expect("Timeout waiting for pooled connection");
    let games = {
        use crawl_model::db_schema::games::dsl::*;
        games
            .filter(species_id.eq(*species as i64))
            .filter(background_id.eq(*bg as i64))
            .filter(god_id.eq(*god as i64))
            .order(score.desc())
            .limit(100)
            .load::<crawl_model::db_model::Game>(&*connection)
            .expect("Error loading games")
    };
    let formatted_games = games.into_iter().map(|x| x.into() ).collect();
    let context = IndexContext { games: formatted_games };
    Template::render("index", &context)
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

#[get("/<file..>", rank = 4)]
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
        .mount("/", routes![hiscores, files, best_species, best_bg, best_combo, best_god, best_combo_inverted, best_combo_and_god, best_player])
        .manage(pool)
        .attach(Template::fairing())
        .launch();
}
