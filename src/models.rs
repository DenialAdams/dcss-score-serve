use super::schema::morgues;

#[derive(Queryable, Serialize)]
pub struct DbMorgue {
    pub file_name: String,
    pub name: String,
    pub version: String,
    pub score: i64,
    pub race: i64,
    pub background: i64,
}
