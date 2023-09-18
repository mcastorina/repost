use crate::db::{
    self,
    models::{Request, Variable},
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

    pub async fn create_variable(&self, args: CreateVariableArgs) -> Result<()> {
        for (env, val) in args.env_vals {
            let var = Variable::new(&args.name, env, val, args.is_secret, &args.source);
            var.save(self.db.pool()).await?;
        }
        Ok(())
    }

    pub async fn delete_requests(&self, args: DeleteRequestsArgs) -> Result<()> {
        for name in args.names {
            sqlx::query("DELETE FROM requests WHERE name = ?")
                .bind(name)
                .execute(self.db.pool())
                .await?;
        }
        Ok(())
    }

    pub async fn delete_variables(&self, args: DeleteVariablesArgs) -> Result<()> {
        // TODO: display deleted variables
        for name_or_id in args.name_or_ids.iter() {
            let vars: Vec<i32> =
                sqlx::query_scalar("SELECT id FROM variables WHERE name = ?1 OR id = ?1")
                    .bind(name_or_id)
                    .fetch_all(self.db.pool())
                    .await
                    .unwrap_or_else(|_| Vec::new());
            if vars.len() == 0 {
                eprintln!("[!] No variable with name or ID: '{}'", name_or_id);
                continue;
            }
            sqlx::query(&format!(
                "DELETE FROM variables WHERE id IN ({})",
                vars.into_iter()
                    .map(|i| i.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            ))
            .execute(self.db.pool())
            .await?;
        }
        Ok(())
    }

    pub async fn get_requests(&self, _args: GetRequestsArgs) -> Result<Vec<Request>> {
        let reqs = db::query_as_request!(
            sqlx::query_as("SELECT * FROM requests")
                .fetch_all(self.db.pool())
                .await?
        );
        Ok(reqs)
    }

    pub async fn get_variables(&self, _args: GetVariablesArgs) -> Result<Vec<Variable>> {
        let vars = db::query_as_variable!(
            sqlx::query_as("SELECT * FROM variables")
                .fetch_all(self.db.pool())
                .await?
        );
        Ok(vars)
    }

    pub async fn get_environments(&self, _args: GetEnvironmentsArgs) -> Result<Vec<String>> {
        let envs: Vec<String> = sqlx::query_scalar("SELECT DISTINCT env FROM variables")
            .fetch_all(self.db.pool())
            .await?;
        Ok(envs)
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

#[derive(Debug)]
pub struct CreateVariableArgs {
    pub name: String,
    pub env_vals: Vec<(String, String)>,
    pub is_secret: bool,
    pub source: String,
}

#[derive(Debug)]
pub struct GetRequestsArgs {
    pub filters: Vec<String>,
}

#[derive(Debug)]
pub struct GetVariablesArgs {
    pub filters: Vec<String>,
}

#[derive(Debug)]
pub struct GetEnvironmentsArgs {
    pub filters: Vec<String>,
}

#[derive(Debug)]
pub struct DeleteVariablesArgs {
    pub name_or_ids: Vec<String>,
}

#[derive(Debug)]
pub struct DeleteRequestsArgs {
    pub names: Vec<String>,
}
