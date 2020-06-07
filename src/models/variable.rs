use rusqlite::{Connection, Result, NO_PARAMS};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Variable {
    name: String,
    env_vals: HashMap<String, String>,
}

impl Variable {
    pub fn new(name: String, env_vals: Vec<(String, String)>) -> Result<Variable, String> {
        let mut env_vals_map = HashMap::new();
        for env_val in env_vals {
            let (env, val) = env_val;
            if env_vals_map.contains_key(&env) {
                return Err(String::from("multiple values provided for an environment"));
            }
            env_vals_map.insert(env, val);
        }
        Ok(Variable {
            name,
            env_vals: env_vals_map,
        })
    }
    pub fn save(&self) -> Result<(), String> {
        let connection = Connection::open("test.db").unwrap();

        // TODO: don't use unwrap
        connection
            .execute(
                "CREATE TABLE IF NOT EXISTS variables (name TEXT, env TEXT, val TEXT);",
                NO_PARAMS,
            )
            .unwrap();
        for (env, val) in &self.env_vals {
            connection
                .execute(
                    "INSERT INTO variables (name, env, val) VALUES (?1, ?2, ?3); ",
                    &[&self.name, &env, &val],
                )
                .unwrap();
        }
        Ok(())
    }
}
