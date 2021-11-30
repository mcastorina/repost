use sqlx::migrate::MigrateDatabase;
use sqlx::{self, Error, Sqlite, SqlitePool};

// * global DB pool that can be referenced for inline queries
// * easily and safely swap DB pool (i.e. workspace)

struct Db {
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
        Ok(Self {
            name: name.to_string(),
            path: path.to_string(),
            pool: Db::load_pool(path).await?,
        })
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
}

#[cfg(test)]
mod test {
    use super::Db;

    #[tokio::test]
    async fn test_db_creation() {
        Db::new("testdb", "file:memdb?mode=memory&cache=shared")
            .await
            .expect("could not create database");
    }
}
