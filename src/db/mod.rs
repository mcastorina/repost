pub mod models;

use sqlx::migrate::MigrateDatabase;
use sqlx::{self, Error, Sqlite, SqlitePool};

use std::path::{Path, PathBuf};

/// Db object for describing the current workspace and storing all
/// data in. This struct uses a sqlite database to store objects to.
#[derive(Clone)]
pub struct Db {
    /// Name of the workspace
    pub name: String,

    /// Path to sqlite database file
    // TODO: use PathBuf (blocked by custom Errors)
    path: String,

    /// Pool of connections
    pool: SqlitePool,
}

impl Db {
    /// Open new connection to `path` and create it if it does
    /// not exist.
    pub async fn new<T, U>(name: T, path: U) -> Result<Self, Error>
    where
        T: Into<String>,
        U: Into<String>,
    {
        let path = path.into();
        let mut db = Self {
            pool: Db::load_pool(&path).await?,
            name: name.into(),
            path,
        };
        db.create_tables().await?;
        Ok(db)
    }

    /// Set the workspace to `name` and create `name.db` if it does
    /// not exist.
    pub async fn set_workspace(&mut self, name: &str) -> Result<(), Error> {
        *self = Self::new(name, format!("{}.db", name)).await?;
        Ok(())
    }

    /// Return a reference to the connection pool for executing sqlite
    /// statements.
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Try to load the connection pool from a path to the sqlite
    /// database file. This function creates the database file if
    /// it does not exist.
    async fn load_pool(path: &str) -> Result<SqlitePool, Error> {
        if !Sqlite::database_exists(path).await? {
            Sqlite::create_database(path).await?
        }
        Ok(SqlitePool::connect(path).await?)
    }

    /// Try to create all tables required in the database file.
    async fn create_tables(&self) -> Result<(), Error> {
        sqlx::query(
            "
            CREATE TABLE IF NOT EXISTS environments (name TEXT PRIMARY KEY NOT NULL);
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

macro_rules! query_as_environment {
    ($query:expr) => {{
        let got: Vec<crate::db::models::DbEnvironment> = $query;
        crate::db::vec_into!(got, crate::db::models::Environment)
    }};
}
pub(crate) use query_as_environment;

macro_rules! query_as_variable {
    ($query:expr) => {{
        let got: Vec<crate::db::models::DbVariable> = $query;
        crate::db::vec_into!(got, crate::db::models::Variable)
    }};
}
pub(crate) use query_as_variable;

macro_rules! query_as_request {
    ($query:expr) => {{
        let got: Vec<crate::db::models::DbRequest> = $query;
        crate::db::vec_into!(got, crate::db::models::Request)
    }};
}
pub(crate) use query_as_request;

// Convert a Vector<DbObject> into a Vector<Object>
// and log errors to stderr, as this indicates a corrupted
// or improperly migrated database.
macro_rules! vec_into {
    ($got:expr, $kind:ty) => {
        $got.into_iter()
            .filter_map(|db_obj| {
                let obj: Result<$kind, _> = db_obj.try_into();
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
        Db::new("testdb", "file:memdb?mode=memory&cache=shared")
            .await
            .expect("could not create database")
    }

    #[tokio::test]
    async fn test_db_creation() {
        // test_db will panic on error
        test_db().await;
    }

    #[tokio::test]
    async fn test_env_get_set() {
        let db = test_db().await;
        let env = Environment::new("foo");
        env.save(db.pool()).await.expect("could not set");

        let got: Environment = sqlx::query_as("SELECT * FROM environments")
            .fetch_one(db.pool())
            .await
            .expect("could not get");
        assert_eq!(got, env);
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
