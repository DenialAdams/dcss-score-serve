#![recursion_limit = "128"]
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
use dotenv::dotenv;
use rocket::response::NamedFile;
use rocket::State;
use rocket_contrib::Template;
use std::ops::Deref;
use std::path::{Path, PathBuf};

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

enum SortOption {
   Longest,
   Shortest,
   New,
   Score,
   Turns,
}

impl<'a> rocket::request::FromFormValue<'a> for SortOption {
   type Error = ();

   fn from_form_value(param: &'a rocket::http::RawStr) -> Result<SortOption, ()> {
      match param.percent_decode_lossy().to_ascii_lowercase().as_ref() {
         "longest" => Ok(SortOption::Longest),
         "shortest" => Ok(SortOption::Shortest),
         "new" => Ok(SortOption::New),
         "score" => Ok(SortOption::Score),
         "turns" => Ok(SortOption::Turns),
         _ => Err(()),
      }
   }

   fn default() -> Option<SortOption> {
      Some(SortOption::Score)
   }
}

#[derive(FromForm)]
struct GameQuery {
   god: Option<God>,
   background: Option<Background>,
   species: Option<Species>,
   name: Option<String>,
   runes: Option<i64>,
   victory: Option<bool>,
   sort_by: SortOption,
}

impl Default for GameQuery {
   fn default() -> GameQuery {
      GameQuery {
         god: None,
         background: None,
         species: None,
         name: None,
         runes: None,
         victory: None,
         sort_by: SortOption::Score,
      }
   }
}

#[derive(Serialize)]
struct IndexContext {
   games: Vec<FormattedGame>,
   count: i64,
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
   pub duration: String,
   pub turns: i64,
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
   pub death_spot: String,
   pub num_runes: i64,
}

fn seconds_to_humantime(mut seconds: i64) -> String {
   let mut hours = 0;
   let mut minutes = 0;
   while seconds >= 3600 {
      seconds -= 3600;
      hours += 1;
   }
   while seconds >= 60 {
      seconds -= 60;
      minutes += 1;
   }
   if hours > 0 {
      format!("{} hours, {} minutes, {} seconds", hours, minutes, seconds)
   } else if minutes > 0 {
      format!("{} minutes, {} seconds", minutes, seconds)
   } else {
      format!("{} seconds", seconds)
   }
}

impl From<crawl_model::db_model::Game> for FormattedGame {
   fn from(game: crawl_model::db_model::Game) -> FormattedGame {
      let species = unsafe { std::mem::transmute::<i64, crawl_model::data::Species>(game.species_id) };
      let background = unsafe { std::mem::transmute::<i64, crawl_model::data::Background>(game.background_id) };
      let god = unsafe { std::mem::transmute::<i64, crawl_model::data::God>(game.god_id) };
      let real_name = match game.name.as_str() {
         "brick" => "Richard",
         "Peen" | "paul" => "Paul",
         "max" | "PunishedMax" | "OgreStreak" => "Max",
         "daddy" | "fuckboy3000" | "peepeedarts" => "James",
         "sweetBro" => "Luca",
         "hellaJeff" | "bigBootyJudy" => "Ben H",
         "Richard" | "BoonShekel" | "THEBLIMP" | "xXBloodSuckerXx" => "Ben S",
         "bobjr93" => "Brennan",
         "jish" => "Josh S",
         "GrapeApe" => "Mason C.",
         "Doomlord5" => "Dan",
         "MikeyBoy" => "Mike",
         "BigSweetPP" => "Seth",
         "Idyll" => "Emma",
         _ => "?",
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
         victory: victory,
         duration: seconds_to_humantime(game.dur),
         turns: game.turn,
      }
   }
}

#[get("/")]
fn hiscores(state: State<DatabasePool>) -> Template {
   hi_query(state, GameQuery::default())
}

#[get("/?<game_query>")]
fn hi_query(state: State<DatabasePool>, game_query: GameQuery) -> Template {
   let connection = state.get().expect("Timeout waiting for pooled connection");
   let games = {
      use crawl_model::db_schema::games::dsl::*;
      let mut expression = games.into_boxed();
      match game_query.sort_by {
         SortOption::Shortest => {
            expression = expression.order(dur.asc());
         }
         SortOption::Longest => {
            expression = expression.order(dur.desc());
         }
         SortOption::New => {
            expression = expression.order(end.desc());
         }
         SortOption::Score => {
            expression = expression.order(score.desc());
         }
         SortOption::Turns => {
            expression = expression.order(turn.asc());
         }
      }
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
         expression = expression.filter(name.eq(qname));
      }
      if let Some(nrunes) = game_query.runes {
         expression = expression.filter(runes.eq(nrunes));
      }
      if let Some(victory) = game_query.victory {
         expression = match victory {
            true => expression.filter(tmsg.eq("escaped with the Orb")),
            false => expression.filter(tmsg.ne("escaped with the Orb")),
         };
      }
      expression
         .limit(100)
         .load::<crawl_model::db_model::Game>(&*connection)
         .expect("Error loading games")
   };
   let formatted_games = games.into_iter().map(|x| x.into()).collect();
   let games_count: i64 = {
      use crawl_model::db_schema::games::dsl::*;
      games.count().get_result(&*connection).expect("Error loading games")
   };
   let context = IndexContext {
      games: formatted_games,
      count: games_count,
   };
   Template::render("index", &context)
}

fn get_user_context(state: State<DatabasePool>, name_param: Option<String>) -> UserContext {
   fn get_query<'a>(
      name_param: Option<&'a String>,
   ) -> crawl_model::db_schema::games::BoxedQuery<'a, diesel::sqlite::Sqlite> {
      use crawl_model::db_schema::games::dsl::*;
      if let Some(val) = name_param {
         games.filter(name.eq(val)).into_boxed()
      } else {
         games.into_boxed()
      }
   }
   let connection = state.get().expect("Timeout waiting for pooled connection");
   let num_games: i64 = {
      get_query(name_param.as_ref())
         .count()
         .get_result(&*connection)
         .expect("Error loading games")
   };
   let num_wins: i64 = {
      use crawl_model::db_schema::games::dsl::*;
      get_query(name_param.as_ref())
         .filter(tmsg.eq("escaped with the Orb"))
         .count()
         .get_result(&*connection)
         .expect("Error loading games")
   };
   let num_runes: i64 = {
      use diesel::dsl::sql;
      use diesel::sql_types::Double;
      get_query(name_param.as_ref())
         .select(sql::<Double>("SUM(games.runes)"))
         .first(&*connection)
         .optional()
         .expect("Error loading games")
         .unwrap_or(0.0) as i64
   };
   let fav_bg = {
      let fav_bg_id: Option<i64> = {
         use crawl_model::db_schema::games::dsl::*;
         use diesel::dsl::count;
         get_query(name_param.as_ref())
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
         get_query(name_param.as_ref())
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
         get_query(name_param.as_ref())
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
         get_query(name_param.as_ref())
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
         get_query(name_param.as_ref())
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
   UserContext {
      fav_background: fav_bg,
      fav_species: fav_species,
      fav_god: fav_god,
      games: num_games,
      wins: num_wins,
      winrate: format!("{:.2}", (num_wins as f64 / num_games as f64) * 100.0),
      name: name_param.unwrap_or_else(|| "Server".into()),
      nemesis: fav_nemesis,
      death_spot: fav_death_spot,
      num_runes: num_runes,
   }
}

#[get("/u/<name_param>")]
fn user(state: State<DatabasePool>, name_param: String) -> Template {
   let context = get_user_context(state, Some(name_param));
   Template::render("user", &context)
}

#[get("/everyone")]
fn everyone(state: State<DatabasePool>) -> Template {
   let context = get_user_context(state, None);
   Template::render("user", &context)
}

#[get("/deaths")]
fn deaths(state: State<DatabasePool>) -> Template {
   let connection = state.get().expect("Timeout waiting for pooled connection");
   let deaths: Vec<(String, i64)> = {
      use crawl_model::db_schema::games::dsl::*;
      use diesel::dsl::sql;
      use diesel::sql_types::BigInt;
      games
         .select((tmsg, sql::<BigInt>("COUNT(games.tmsg)")))
         .order(sql::<BigInt>("COUNT(games.tmsg)").desc())
         .group_by(tmsg)
         .limit(100)
         .load::<_>(&*connection)
         .expect("Error loading games")
   };
   let formatted_items = deaths
      .into_iter()
      .map(|x| FormattedFreqItem {
         value: x.0,
         frequency: x.1,
      })
      .collect();
   let context = FreqContext {
      name: "Cause of Death",
      items: formatted_items,
   };
   Template::render("frequency", &context)
}

#[get("/places")]
fn places(state: State<DatabasePool>) -> Template {
   let connection = state.get().expect("Timeout waiting for pooled connection");
   let places: Vec<(String, i64)> = {
      use crawl_model::db_schema::games::dsl::*;
      use diesel::dsl::sql;
      use diesel::sql_types::BigInt;
      games
         .select((place, sql::<BigInt>("COUNT(games.place)")))
         .order(sql::<BigInt>("COUNT(games.place)").desc())
         .group_by(place)
         .limit(100)
         .load::<_>(&*connection)
         .expect("Error loading games")
   };
   let formatted_items = places
      .into_iter()
      .map(|x| FormattedFreqItem {
         value: x.0,
         frequency: x.1,
      })
      .collect();
   let context = FreqContext {
      name: "Final Location",
      items: formatted_items,
   };
   Template::render("frequency", &context)
}

#[get("/species")]
fn species(state: State<DatabasePool>) -> Template {
   let connection = state.get().expect("Timeout waiting for pooled connection");
   let species: Vec<(i64, i64)> = {
      use crawl_model::db_schema::games::dsl::*;
      use diesel::dsl::sql;
      use diesel::sql_types::BigInt;
      games
         .select((species_id, sql::<BigInt>("COUNT(games.species_id)")))
         .order(sql::<BigInt>("COUNT(games.species_id)").desc())
         .group_by(species_id)
         .limit(100)
         .load::<_>(&*connection)
         .expect("Error loading games")
   };
   let formatted_items = species
      .into_iter()
      .map(|x| FormattedFreqItem {
         value: format!("{:?}", unsafe {
            std::mem::transmute::<i64, crawl_model::data::Species>(x.0)
         }),
         frequency: x.1,
      })
      .collect();
   let context = FreqContext {
      name: "Species",
      items: formatted_items,
   };
   Template::render("frequency", &context)
}

#[get("/backgrounds")]
fn backgrounds(state: State<DatabasePool>) -> Template {
   let connection = state.get().expect("Timeout waiting for pooled connection");
   let backgrounds: Vec<(i64, i64)> = {
      use crawl_model::db_schema::games::dsl::*;
      use diesel::dsl::sql;
      use diesel::sql_types::BigInt;
      games
         .select((background_id, sql::<BigInt>("COUNT(games.background_id)")))
         .order(sql::<BigInt>("COUNT(games.background_id)").desc())
         .group_by(background_id)
         .limit(100)
         .load::<_>(&*connection)
         .expect("Error loading games")
   };
   let formatted_items = backgrounds
      .into_iter()
      .map(|x| FormattedFreqItem {
         value: format!("{:?}", unsafe {
            std::mem::transmute::<i64, crawl_model::data::Background>(x.0)
         }),
         frequency: x.1,
      })
      .collect();
   let context = FreqContext {
      name: "Background",
      items: formatted_items,
   };
   Template::render("frequency", &context)
}

#[get("/gods")]
fn gods(state: State<DatabasePool>) -> Template {
   let connection = state.get().expect("Timeout waiting for pooled connection");
   let gods: Vec<(i64, i64)> = {
      use crawl_model::db_schema::games::dsl::*;
      use diesel::dsl::sql;
      use diesel::sql_types::BigInt;
      games
         .select((god_id, sql::<BigInt>("COUNT(games.god_id)")))
         .order(sql::<BigInt>("COUNT(games.god_id)").desc())
         .group_by(god_id)
         .limit(100)
         .load::<_>(&*connection)
         .expect("Error loading games")
   };
   let formatted_items = gods
      .into_iter()
      .map(|x| FormattedFreqItem {
         value: format!("{:?}", unsafe {
            std::mem::transmute::<i64, crawl_model::data::God>(x.0)
         }),
         frequency: x.1,
      })
      .collect();
   let context = FreqContext {
      name: "God",
      items: formatted_items,
   };
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
      .mount(
         "/",
         routes![
            hiscores,
            files,
            deaths,
            hi_query,
            species,
            backgrounds,
            gods,
            user,
            places,
            everyone
         ],
      )
      .manage(pool)
      .attach(Template::fairing())
      .launch();
}
