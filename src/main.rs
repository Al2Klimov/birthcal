#![recursion_limit = "275"]

mod cli;

use crate::cli::EnvError;
use cgi::{empty_response, handle, html_response, Request, Response};
use chrono::{Datelike, NaiveDate};
use html::root::Html;
use ical::parser::vcard::component::VcardContact;
use ical::parser::Component;
use percent_encoding_rfc3986::{utf8_percent_encode, NON_ALPHANUMERIC};
use std::collections::BTreeMap;
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

    let srch = match cli::require_noempty_utf8_env("BIRTHCAL_SEARCH") {
        Err(err) => match err.err {
            EnvError::Missing => None,
            _ => {
                eprintln!("{}", err);
                return empty_response(500);
            }
        },
        Ok(url) => Some(url),
    };

    match ureq::get(url.clone()).call() {
        Err(err) => {
            eprintln!("GET {}: {}", url, err);
            empty_response(502)
        }
        Ok(mut resp) => {
            let mut urls_by_mdy_name = BTreeMap::new();

            for i in ical::VcardParser::new(BufReader::new(resp.body_mut().as_reader())) {
                match i {
                    Err(err) => {
                        eprintln!("GET {}: {}", url, err);
                        return empty_response(502);
                    }
                    Ok(mut vcard) => match contact_prop(&mut vcard, "BDAY") {
                        None => {}
                        Some(birthday) => match contact_prop(&mut vcard, "FN") {
                            None => {}
                            Some(name) => {
                                let df = "%Y%m%d";

                                match NaiveDate::parse_from_str(birthday.as_str(), df) {
                                    Err(err) => {
                                        eprintln!("{} is not like {}: {}", birthday, df, err);
                                        return empty_response(502);
                                    }
                                    Ok(date) => {
                                        urls_by_mdy_name.insert(
                                            (date.month(), date.day(), date.year(), name),
                                            contact_prop(&mut vcard, "URL"),
                                        );
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
                            for ((month, day, year, name), url) in urls_by_mdy_name {
                                table.table_row(|tr| {
                                    tr.table_cell(|td| {
                                        td.text(format!("{}-{}-{}", year, month, day))
                                    })
                                    .table_cell(
                                        |td| match url {
                                            Some(url) => td.anchor(|a| {
                                                a.target("_blank").href(url).text(name)
                                            }),
                                            None => match &srch {
                                                Some(url) => td.anchor(|a| {
                                                    a.target("_blank")
                                                        .href(format!(
                                                            "{}{}",
                                                            url,
                                                            utf8_percent_encode(
                                                                name.as_str(),
                                                                NON_ALPHANUMERIC
                                                            )
                                                        ))
                                                        .text(name)
                                                }),
                                                None => td.text(name),
                                            },
                                        },
                                    )
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

fn contact_prop(contact: &mut VcardContact, prop: &'static str) -> Option<String> {
    match contact.get_property_mut(prop) {
        None => None,
        Some(val) => val.value.take(),
    }
}
