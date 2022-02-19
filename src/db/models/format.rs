use comfy_table::{Cell, Table};
use std::fmt::{self, Display, Formatter};
use std::io;

pub const TABLE_FORMAT: &'static str = "||--+-++|    ++++++";

pub trait DisplayTable {
    const HEADER: &'static [&'static str];

    fn build(&self, table: &mut Table);
    fn print(&self) {
        self.write(io::stdout()).expect("could not write to stdout");
    }
    fn write<W: io::Write>(&self, mut w: W) -> Result<(), io::Error> {
        // generate table
        let mut table = Table::new();
        table
            .load_preset(TABLE_FORMAT)
            .set_table_width(80)
            .set_header(Self::HEADER);
        // add rows from the vector
        self.build(&mut table);
        // print a blank line
        writeln!(w)?;
        // indent each row by two spaces
        for line in table.to_string().split('\n') {
            writeln!(w, "  {}", line)?;
        }
        // print a blank line
        writeln!(w)?;
        Ok(())
    }
}

impl<T: DisplayTable> DisplayTable for Vec<T> {
    const HEADER: &'static [&'static str] = T::HEADER;

    fn build(&self, table: &mut Table) {
        self.iter().for_each(|obj| obj.build(table));
    }
}

impl DisplayTable for super::Request {
    const HEADER: &'static [&'static str] = &["name", "method", "url", "headers", "body?"];

    fn build(&self, table: &mut Table) {
        let headers = self
            .headers
            .iter()
            .flat_map(|(k, v)| [k.as_str(), ": ", v.as_str()])
            .fold(String::new(), |s, h| s + h + "\n");
        let headers = headers.trim();
        table.add_row(&[
            &self.name,
            &self.method.to_string(),
            &self.url.to_string(),
            headers,
            &self.body.as_ref().map(|b| b.kind()).unwrap_or_default(),
        ]);
    }
}

impl DisplayTable for super::Environment {
    const HEADER: &'static [&'static str] = &["environment"];

    fn build(&self, table: &mut Table) {
        table.add_row(&[&self.name]);
    }
}

impl DisplayTable for super::Variable {
    const HEADER: &'static [&'static str] = &["id", "name", "environment", "value", "source"];

    fn build(&self, table: &mut Table) {
        table.add_row(&[
            &self.id.map(|id| id.to_string()).unwrap_or_default(),
            &self.name,
            &self.env.name,
            self.value.as_ref().unwrap_or(&String::new()),
            &self.source,
        ]);
    }
}