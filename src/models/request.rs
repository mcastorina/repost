
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

        pub fn save(&self) -> Result<(), &'static str> {
            Err("save not implemented")
        }
    }
