use rusqlite::{Connection, Result, NO_PARAMS};

#[derive(Debug)]
pub struct Request {
    name: String,
    url: String,
    method: String,
    headers: Vec<String>,
    body: Option<String>,
}
impl Request {
    pub fn new(
        name: String,
        url: String,
        method: String,
        headers: Vec<String>,
        body: Option<String>,
    ) -> Result<Request, &'static str> {
        // TODO: validate name is unique
        // TODO: validate method
        // TODO: validate headers
        Ok(Request {
            name,
            url,
            method,
            headers,
            body,
        })
    }

    pub fn save(&self) -> Result<(), String> {
        let connection = Connection::open("test.db").unwrap();

        // TODO: don't use unwrap
        connection.execute(
            "CREATE TABLE IF NOT EXISTS requests (name TEXT, url TEXT, method TEXT, headers TEXT, body TEXT);",
            NO_PARAMS,
        ).unwrap();

        connection
                .execute(
                    "INSERT INTO requests (name, url, method, headers, body) VALUES (?1, ?2, ?3, ?4, ?5); ",
                    &[&self.name, &self.url, &self.method, &self.headers.join(","), self.body.as_ref().unwrap_or(&String::from(""))],
                )
                .unwrap();
        Ok(())
    }
}
