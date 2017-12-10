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
    name: Option<String>,
    victory: bool
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
struct FreqContext<'a> {
    name: &'a str,
    items: Vec<FormattedFreqItem>,
}

#[derive(Serialize)]
struct FormattedFreqItem {
    pub frequency: i64,
    pub value: String,
}

#[derive(Serialize)]
struct UserContext {
    pub fav_species: String,
    pub fav_background: String,
    pub fav_god: String,
    pub wins: i64,
    pub games: i64,
    pub winrate: String,
    pub name: String,
    pub nemesis: String,
    pub death_spot: String
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
            "BigSweetPP" => "Seth",
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
        if game_query.victory {
            expression = expression.filter(tmsg.eq("escaped with the Orb"))
        }
        expression.load::<crawl_model::db_model::Game>(&*connection).expect("Error loading games")
    };
    let formatted_games = games.into_iter().map(|x| x.into() ).collect();
    let context = IndexContext { games: formatted_games };
    Template::render("index", &context)
}

#[get("/u/<name_param>")]
fn user(state: State<DatabasePool>, name_param: String) -> Template {
    let connection = state.get().expect("Timeout waiting for pooled connection");
    let num_games: i64 = {
        use crawl_model::db_schema::games::dsl::*;
        games
            .filter(name.eq(&name_param))
            .count()
            .get_result(&*connection)
            .expect("Error loading games")
    };
    let num_wins: i64 = {
        use crawl_model::db_schema::games::dsl::*;
        games
            .filter(name.eq(&name_param))
            .filter(tmsg.eq("escaped with the Orb"))
            .count()
            .get_result(&*connection)
            .expect("Error loading games")
    };
    let fav_bg = {
        let fav_bg_id: Option<i64> = {
            use crawl_model::db_schema::games::dsl::*;
            use diesel::dsl::count;
            games
                .filter(name.eq(&name_param))
                .order(count(background_id).desc())
                .select(background_id)
                .group_by(background_id)
                .first(&*connection)
                .optional()
                .expect("Error loading games")
        };
        if let Some(bg_id) = fav_bg_id {
            let background = unsafe { std::mem::transmute::<i64, crawl_model::data::Background>(bg_id) };
            format!("{:?}", background)
        } else {
            "N/A".into()
        }
    };
    let fav_species = {
        let fav_species_id: Option<i64> = {
            use crawl_model::db_schema::games::dsl::*;
            use diesel::dsl::count;
            games
                .filter(name.eq(&name_param))
                .order(count(species_id).desc())
                .select(species_id)
                .group_by(species_id)
                .first(&*connection)
                .optional()
                .expect("Error loading games")
        };
        if let Some(species_id) = fav_species_id {
            let species = unsafe { std::mem::transmute::<i64, crawl_model::data::Species>(species_id) };
            format!("{:?}", species)
        } else {
            "N/A".into()
        }
    };
    let fav_god = {
        let fav_god_id: Option<i64> = {
            use crawl_model::db_schema::games::dsl::*;
            use diesel::dsl::count;
            games
                .filter(name.eq(&name_param))
                .filter(god_id.ne(crawl_model::data::God::Atheist as i64))
                .order(count(god_id).desc())
                .select(god_id)
                .group_by(god_id)
                .first(&*connection)
                .optional()
                .expect("Error loading games")
        };
        if let Some(god_id) = fav_god_id {
            let god = unsafe { std::mem::transmute::<i64, crawl_model::data::God>(god_id) };
            format!("{:?}", god)
        } else {
            "N/A".into()
        }
    };
    let fav_nemesis = {
        let fav_nemesis: Option<String> = {
            use crawl_model::db_schema::games::dsl::*;
            use diesel::dsl::count;
            games
                .filter(name.eq(&name_param))
                .filter(tmsg.ne("got out of the dungeon alive"))
                .filter(tmsg.ne("quit the game"))
                .filter(tmsg.ne("safely got out of the dungeon"))
                .order(count(tmsg).desc())
                .select(tmsg)
                .group_by(tmsg)
                .first(&*connection)
                .optional()
                .expect("Error loading games")
        };
        if let Some(nemesis) = fav_nemesis {
            nemesis
        } else {
            "N/A".into()
        }
    };
    let fav_death_spot = {
        let fav_death_spot: Option<String> = {
            use crawl_model::db_schema::games::dsl::*;
            use diesel::dsl::count;
            games
                .filter(name.eq(&name_param))
                .order(count(place).desc())
                .select(place)
                .group_by(place)
                .first(&*connection)
                .optional()
                .expect("Error loading games")
        };
        if let Some(death_spot) = fav_death_spot {
            death_spot
        } else {
            "N/A".into()
        }
    };
    let context = UserContext {
        fav_background: fav_bg,
        fav_species: fav_species,
        fav_god: fav_god,
        games: num_games,
        wins: num_wins,
        winrate: format!("{:.2}", (num_wins as f64 / num_games as f64) * 100.0),
        name: name_param,
        nemesis: fav_nemesis,
        death_spot: fav_death_spot
    };
    Template::render("user", &context)
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
    let formatted_items = deaths.into_iter().map(|x| FormattedFreqItem { value: x.0, frequency: x.1 }).collect();
    let context = FreqContext { name: "Cause of Death", items: formatted_items };
    Template::render("frequency", &context)
}

#[get("/places")]
fn places(state: State<DatabasePool>) -> Template {
    let connection = state.get().expect("Timeout waiting for pooled connection");
    let places: Vec<(String, i64)> = {
        use diesel::dsl::count;
        use diesel::types::BigInt;
        use diesel::dsl::sql;
        use crawl_model::db_schema::games::dsl::*;
        games
            .select((place, sql::<BigInt>("COUNT(games.place)")))
            .order(sql::<BigInt>("COUNT(games.place)").desc())
            .group_by(place)
            .limit(100)
            .load::<_>(&*connection)
            .expect("Error loading games")
    };
    let formatted_items = places.into_iter().map(|x| FormattedFreqItem { value: x.0, frequency: x.1 }).collect();
    let context = FreqContext { name: "Final Location", items: formatted_items };
    Template::render("frequency", &context)
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
    let formatted_items = species.into_iter().map(|x| FormattedFreqItem { value: format!("{:?}", unsafe { std::mem::transmute::<i64, crawl_model::data::Species>(x.0) } ), frequency: x.1 }).collect();
    let context = FreqContext { name: "Species", items: formatted_items };
    Template::render("frequency", &context)
}

#[get("/backgrounds")]
fn backgrounds(state: State<DatabasePool>) -> Template {
    let connection = state.get().expect("Timeout waiting for pooled connection");
    let backgrounds: Vec<(i64, i64)> = {
        use diesel::dsl::count;
        use diesel::types::BigInt;
        use diesel::dsl::sql;
        use crawl_model::db_schema::games::dsl::*;
        games
            .select((background_id, sql::<BigInt>("COUNT(games.background_id)")))
            .order(sql::<BigInt>("COUNT(games.background_id)").desc())
            .group_by(background_id)
            .limit(100)
            .load::<_>(&*connection)
            .expect("Error loading games")
    };
    let formatted_items = backgrounds.into_iter().map(|x| FormattedFreqItem { value: format!("{:?}", unsafe { std::mem::transmute::<i64, crawl_model::data::Background>(x.0) } ), frequency: x.1 }).collect();
    let context = FreqContext { name: "Background", items: formatted_items };
    Template::render("frequency", &context)
}

#[get("/gods")]
fn gods(state: State<DatabasePool>) -> Template {
    let connection = state.get().expect("Timeout waiting for pooled connection");
    let gods: Vec<(i64, i64)> = {
        use diesel::dsl::count;
        use diesel::types::BigInt;
        use diesel::dsl::sql;
        use crawl_model::db_schema::games::dsl::*;
        games
            .select((god_id, sql::<BigInt>("COUNT(games.god_id)")))
            .order(sql::<BigInt>("COUNT(games.god_id)").desc())
            .group_by(god_id)
            .limit(100)
            .load::<_>(&*connection)
            .expect("Error loading games")
    };
    let formatted_items = gods.into_iter().map(|x| FormattedFreqItem { value: format!("{:?}", unsafe { std::mem::transmute::<i64, crawl_model::data::God>(x.0) } ), frequency: x.1 }).collect();
    let context = FreqContext { name: "God", items: formatted_items };
    Template::render("frequency", &context)
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
        .mount("/", routes![hiscores, files, deaths, hi_query, species, backgrounds, gods, user, places])
        .manage(pool)
        .attach(Template::fairing())
        .launch();
}
