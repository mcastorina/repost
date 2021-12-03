pub mod models;

use sqlx::migrate::MigrateDatabase;
use sqlx::{self, Error, Sqlite, SqlitePool};

/// Db object for describing the current workspace and storing all
/// data in. This struct uses a sqlite database to store objects to.
pub struct Db {
    /// Name of the workspace
    pub name: String,

    /// Path to sqlite database file
    path: String,

    /// Pool of connections
    pool: SqlitePool,
}

impl Db {
    /// Open new connection to `path` and create it if it does
    /// not exist.
    pub async fn new(name: &str, path: &str) -> Result<Self, Error> {
        let mut db = Self {
            name: name.to_string(),
            path: path.to_string(),
            pool: Db::load_pool(path).await?,
        };
        db.create_tables().await?;
        Ok(db)
    }

    /// Set the workspace to `name` and create `name.db` if it does
    /// not exist.
    pub async fn set_workspace(&mut self, name: &str) -> Result<(), Error> {
        *self = Self::new(name, &format!("{}.db", name)).await?;
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
        if !Sqlite::database_exists(&path).await? {
            Sqlite::create_database(&path).await?
        }
        Ok(SqlitePool::connect(&path).await?)
    }

    /// Try to create all tables required in the database file.
    async fn create_tables(&self) -> Result<(), Error> {
        sqlx::query(
            "
            CREATE TABLE IF NOT EXISTS environments (name TEXT PRIMARY KEY NOT NULL);
            ",
        )
        .execute(self.pool())
        .await?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::models::Environment;
    use super::Db;

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
        env.save(db.pool()).await.expect("could not get");

        let got: Environment = sqlx::query_as("SELECT * FROM environments")
            .fetch_one(db.pool())
            .await
            .expect("could not get");
        assert_eq!(got, env);
    }
}
