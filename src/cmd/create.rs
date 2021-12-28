use super::models::{Environment, Variable};
use crate::db::models as db;
use crate::db::Db;
use builder_pattern::Builder;
use sqlx::Error;

/// Create an environment and save it to the database. This function
/// returns an error if the environment already exists.
pub async fn environment(db: &Db, name: &str) -> Result<(), Error> {
    let env: db::Environment = Environment::new(name).into();
    env.save(db.pool()).await
}

pub async fn variable(db: &Db, name: &str, env: &str, value: &str) -> Result<(), Error> {
    let var: db::Variable = Variable::new(name, env, value, "user").into();
    var.save(db.pool()).await
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
        // environments are unique by name
        assert!(super::environment(&db, "foo").await.is_err());
    }

    #[tokio::test]
    async fn test_create_variable() {
        let db = test_db().await;
        super::variable(&db, "foo", "env", "bar")
            .await
            .expect("create::variable failed");
        // variables are unique by id
        assert!(super::variable(&db, "foo", "env", "bar").await.is_ok());
    }
}
