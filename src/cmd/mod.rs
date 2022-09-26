use crate::db::{
    self,
    models::{DisplayTable, Request},
    Db,
};
use crate::error::Result;
use reqwest::Method;

pub struct Cmd<'db> {
    db: &'db Db,
}

impl<'db> Cmd<'db> {
    pub fn new(db: &'db Db) -> Self {
        Self { db }
    }

    pub async fn create_request(&self, args: CreateRequestArgs) -> Result<()> {
        let req = Request::new(args.name, args.method, args.url).headers(args.headers);
        let req = match args.body {
            Some(body) => req.body(body),
            None => req,
        };

        req.save(self.db.pool()).await?;
        Ok(())
    }

    pub async fn print_requests(&self) -> Result<()> {
        let reqs = db::query_as_request!(
            sqlx::query_as("SELECT * FROM requests")
                .fetch_all(self.db.pool())
                .await?
        );
        reqs.print();
        Ok(())
    }
}

#[derive(Debug)]
pub struct CreateRequestArgs {
    pub name: String,
    pub url: String,
    pub method: Method,
    pub headers: Vec<(String, String)>,
    pub body: Option<String>,
}
