use super::PrintableTable;
use super::{DbObject, InputOption, OutputOption, Request};
use crate::error::{Error, ErrorKind, Result};
use comfy_table::Cell;
use reqwest::blocking;
use reqwest::Method;

#[derive(Debug)]
pub struct RequestRunner {
    reqwests: Vec<blocking::Request>,
    // TODO: preactions / postactions
}

impl RequestRunner {
    pub fn new(r: Request) -> Result<RequestRunner> {
        Ok(RequestRunner {
            reqwests: RequestRunner::create_reqwests(r)?,
        })
    }
    pub fn run(self) -> Result<()> {
        for reqw in self.reqwests {
            blocking::Client::new().execute(reqw)?;
        }
        Ok(())
    }

    fn create_reqwests(req: Request) -> Result<Vec<blocking::Request>> {
        let input_opts = req.input_options();
        let missing_opts: Vec<_> = input_opts
            .iter()
            .filter(|opt| opt.values().len() == 0)
            .map(|opt| String::from(opt.option_name()))
            .collect();
        if missing_opts.len() > 0 {
            // All input options are required
            return Err(Error::new(ErrorKind::MissingOptions(missing_opts)));
        }

        if input_opts.len() == 0 {
            let mut req = req.clone();
            return Ok(vec![RequestRunner::create_reqwest(req)?]);
        }

        let mut reqwests = Vec::new();
        let opts: Vec<_> = input_opts.iter().map(|opt| opt.values()).collect();

        for i in 0..opts.iter().map(|x| x.len()).max().unwrap_or(0) {
            let opt_values: Vec<&str> = opts.iter().map(|v| v[i % v.len()]).collect();
            let mut opts = input_opts.clone();
            let mut req = req.clone();
            for (opt, opt_value) in opts.iter_mut().zip(opt_values) {
                req.set_option(opt.option_name(), vec![opt_value])?;
            }
            reqwests.push(RequestRunner::create_reqwest(req)?);
        }
        Ok(reqwests)
    }
    fn create_reqwest(mut req: Request) -> Result<blocking::Request> {
        let client = blocking::Client::new();
        req.replace_input_options()?;

        let mut builder = client.request(req.take_method(), req.url());
        // add headers
        for header in req.headers().iter() {
            let mut items = header.splitn(2, ":");
            let (header, value) = (items.next(), items.next());
            if header.and(value).is_none() {
                continue;
            }
            builder = builder.header(header.unwrap(), value.unwrap());
        }
        // add body
        if let Some(x) = req.take_body() {
            builder = builder.body(x);
        }

        Ok(builder.build()?)
    }
}

impl Default for RequestRunner {
    fn default() -> Self {
        RequestRunner{
            reqwests: vec![],
        }
    }
}

impl PrintableTable for RequestRunner {
    fn get_header(&self) -> Vec<Cell> {
        // TODO: table of option / values
        vec![
            Cell::new("id"),
            Cell::new("url"),
        ]
    }
    fn get_rows(&self) -> Vec<Vec<Cell>> {
        let mut rows = vec![];
        for (i, reqw) in self.reqwests.iter().enumerate() {
            let row = vec![
                Cell::new(i),
                Cell::new(reqw.url()),
            ];
            rows.push(row);
        }
        rows
    }
}
