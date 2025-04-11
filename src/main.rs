#![recursion_limit = "512"]

mod cli;

use cgi::{empty_response, handle, html_response, Request, Response};
use chrono::{Datelike, NaiveDate};
use html::root::Html;
use ical::parser::vcard::component::VcardContact;
use ical::parser::Component;
use std::collections::BTreeSet;
use std::io::BufReader;

fn main() {
    handle(handler)
}

fn handler(_: Request) -> Response {
    let url = match cli::require_noempty_utf8_env("BIRTHCAL_CARDS") {
        Err(err) => {
            eprintln!("{}", err);
            return empty_response(500);
        }
        Ok(url) => url,
    };

    match ureq::get(url.clone()).call() {
        Err(err) => {
            eprintln!("GET {}: {}", url, err);
            empty_response(502)
        }
        Ok(mut resp) => {
            let mut names_by_mdy = BTreeSet::new();

            for i in ical::VcardParser::new(BufReader::new(resp.body_mut().as_reader())) {
                match i {
                    Err(err) => {
                        eprintln!("GET {}: {}", url, err);
                        return empty_response(502);
                    }
                    Ok(vcard) => match contact_prop(&vcard, "BDAY") {
                        None => {}
                        Some(birthday) => match contact_prop(&vcard, "FN") {
                            None => {}
                            Some(name) => {
                                let df = "%Y%m%d";

                                match NaiveDate::parse_from_str(
                                    birthday.replace("-", "").as_str(),
                                    df,
                                ) {
                                    Err(err) => {
                                        eprintln!("{} is not like {}: {}", birthday, df, err);
                                        return empty_response(502);
                                    }
                                    Ok(date) => {
                                        names_by_mdy.replace((
                                            date.month(),
                                            date.day(),
                                            date.year(),
                                            name.clone(),
                                        ));
                                    }
                                }
                            }
                        },
                    },
                }
            }

            html_response(
                200,
                Html::builder()
                    .body(|body| {
                        body.table(|table| {
                            for (month, day, year, name) in names_by_mdy {
                                table.table_row(|tr| {
                                    tr.table_cell(|td| {
                                        td.text(format!("{}-{}-{}", year, month, day))
                                    })
                                    .table_cell(|td| td.text(name))
                                });
                            }
                            table
                        })
                    })
                    .build()
                    .to_string(),
            )
        }
    }
}

fn contact_prop<'a>(contact: &'a VcardContact, prop: &'static str) -> Option<&'a String> {
    match contact.get_property(prop) {
        None => None,
        Some(val) => match &val.value {
            None => None,
            Some(v) => Some(v),
        },
    }
}
