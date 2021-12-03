use crate::db::models::Environment;
use crate::db::Db;
use builder_pattern::Builder;
use sqlx::Error;

/// Create an environment and save it to the database. This function
/// returns an error if the environment already exists.
pub async fn environment(db: &Db, name: &str) -> Result<(), Error> {
    Environment::new(name).save(db.pool()).await
}

#[cfg(test)]
mod test {
    use crate::db::Db;

    // create an in-memory database for testing
    async fn test_db() -> Db {
        Db::new("testdb", "file:memdbCreate?mode=memory&cache=shared")
            .await
            .expect("could not create database")
    }

    #[tokio::test]
    async fn test_create_environment() {
        let db = test_db().await;
        super::environment(&db, "foo")
            .await
            .expect("create::environment failed");
        assert!(super::environment(&db, "foo").await.is_err());
    }
}
