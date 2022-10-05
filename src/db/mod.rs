pub mod models;
pub use models::DisplayTable;

use std::path::Path;

use sqlx::{
    self,
    migrate::MigrateDatabase,
    sqlite::{SqliteConnectOptions, SqliteJournalMode},
    Sqlite, SqlitePool,
};

use crate::error::{Error, Result};

/// Db object for describing the current workspace and storing all
/// data in. This struct uses a sqlite database to store objects to.
#[derive(Clone)]
pub struct Db {
    /// Sqlite database path string
    path: String,

    /// Pool of connections
    pool: SqlitePool,
}

impl Db {
    const PLAYGROUND: &'static str = "file:memdb?mode=memory&cache=shared";

    /// Open new connection to `path` and create it if it does
    /// not exist.
    pub async fn new<P>(path: P) -> Result<Self>
    where
        P: Into<String>,
    {
        let path = path.into();
        let db = Self {
            pool: Db::load_pool(&path).await?,
            path,
        };
        db.create_tables().await?;
        Ok(db)
    }

    /// Creates an in-memory database for experimentation.
    pub async fn new_playground() -> Result<Self> {
        Self::new(Self::PLAYGROUND).await
    }

    /// Return a reference to the connection pool for executing sqlite
    /// statements.
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Return the name of the database.
    pub fn name(&self) -> &str {
        Self::name_of(&self.path)
    }

    pub fn name_of<S: AsRef<str> + ?Sized>(path: &S) -> &str {
        let path = path.as_ref();
        if path == Self::PLAYGROUND {
            return "playground";
        }
        match Path::new(path).file_name().and_then(|o| o.to_str()) {
            Some(s) => s.strip_suffix(".db").unwrap_or(s),
            None => path,
        }
    }

    /// Try to load the connection pool from a path to the sqlite
    /// database file. This function creates the database file if
    /// it does not exist. In addition to filepaths, path may also
    /// represent an in-memory connection such as `file:db?mode=memory`.
    async fn load_pool(path: &str) -> Result<SqlitePool> {
        if !Sqlite::database_exists(path).await? {
            Sqlite::create_database(path).await?
        }
        let options = SqliteConnectOptions::new()
            .filename(path)
            .journal_mode(SqliteJournalMode::Off);
        Ok(SqlitePool::connect_with(options).await?)
    }

    /// Try to create all tables required in the database file.
    async fn create_tables(&self) -> Result<()> {
        sqlx::query(
            "
            CREATE TABLE IF NOT EXISTS variables (
                id          INTEGER PRIMARY KEY,
                name        TEXT NOT NULL,
                env         TEXT NOT NULL,
                value       TEXT,
                source      TEXT NOT NULL,
                timestamp   TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS requests (
                name        TEXT PRIMARY KEY,
                method      TEXT NOT NULL,
                url         TEXT NOT NULL,
                headers     TEXT,
                body_kind   TEXT,
                body        BLOB
            );
            ",
        )
        .execute(self.pool())
        .await?;
        Ok(())
    }
}

// Macros to make querying more ergonomic
// TODO: make generic over a type

/// Convenience macro for querying for variables and converting
/// from a Vec<DbVariable> to a Vec<Variable>.
macro_rules! query_as_variable {
    ($query:expr) => {{
        let got: Vec<crate::db::models::DbVariable> = $query;
        crate::db::vec_into!(got, crate::db::models::Variable)
    }};
}
pub(crate) use query_as_variable;

/// Convenience macro for querying for requests and converting
/// from a Vec<DbRequest> to a Vec<Request>.
macro_rules! query_as_request {
    ($query:expr) => {{
        let got: Vec<crate::db::models::DbRequest> = $query;
        crate::db::vec_into!(got, crate::db::models::Request)
    }};
}
pub(crate) use query_as_request;

/// Convert a Vector<DbObject> into a Vector<Object> and log errors to stderr, as this indicates a
/// corrupted or improperly migrated database. Requires importing std::convert::TryInto trait.
macro_rules! vec_into {
    ($got:expr, $kind:ty) => {
        $got.into_iter()
            .filter_map(|db_obj| {
                let obj: std::result::Result<$kind, _> = db_obj.try_into();
                match obj {
                    Ok(obj) => Some(obj),
                    Err(err) => {
                        eprintln!("Error converting DbObject into Object: {:?}", err);
                        None
                    }
                }
            })
            .collect::<Vec<_>>()
    };
}
pub(crate) use vec_into;

#[cfg(test)]
mod test {
    use super::models::{DbRequest, DbVariable, Environment, Request, Variable};
    use super::Db;
    use std::convert::TryInto;

    // create an in-memory database for testing
    async fn test_db() -> Db {
        Db::new("file:memdb?mode=memory&cache=shared")
            .await
            .expect("could not create database")
    }

    #[tokio::test]
    async fn test_db_creation() {
        // test_db will panic on error
        test_db().await;
    }

    #[tokio::test]
    async fn test_var_get_set() {
        let db = test_db().await;
        let var = Variable::new("foo", "env", "value", "source");
        var.clone().save(db.pool()).await.expect("could not set");

        let got: DbVariable = sqlx::query_as("SELECT * FROM variables")
            .fetch_one(db.pool())
            .await
            .expect("could not get");
        let mut got: Variable = got.into();

        // when we get a variable from the database, it's ID will be set
        assert!(got.id.is_some());

        // set to None for the rest of the comparison
        got.id = None;
        assert_eq!(got, var);
    }

    #[tokio::test]
    async fn test_req_get_set() {
        let db = test_db().await;
        let req = Request::new("name", "method", "url")
            .header("foo", "bar")
            .body("baz");
        req.clone().save(db.pool()).await.expect("could not set");

        let got: DbRequest = sqlx::query_as("SELECT * FROM requests")
            .fetch_one(db.pool())
            .await
            .expect("could not get");
        let got: Request = got.try_into().expect("db data did not parse");
        assert_eq!(got, req);
    }
}
