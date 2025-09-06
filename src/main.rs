#![recursion_limit = "512"]

mod cli;

use crate::cli::EnvError;
use cgi::{Request, Response, empty_response, handle, html_response};
use html::root::Html;
use html::tables::builders::TableCellBuilder;
use ical::parser::Component;
use ical::parser::vcard::component::VcardContact;
use percent_encoding_rfc3986::{NON_ALPHANUMERIC, utf8_percent_encode};
use regex::{Captures, Regex};
use std::collections::BTreeMap;
use std::io::BufReader;
use std::str::FromStr;

fn main() {
    handle(handler)
}

fn handler(_: Request) -> Response {
    let yyyymmdd = Regex::new(r"\A(--|[0-9]{4})([0-9]{2})([0-9]{2})\z").unwrap();

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
            let mut urls_by_name = BTreeMap::new();

            for i in ical::VcardParser::new(BufReader::new(resp.body_mut().as_reader())) {
                match i {
                    Err(err) => {
                        eprintln!("GET {}: {}", url, err);
                        return empty_response(502);
                    }
                    Ok(mut vcard) => match contact_prop(&mut vcard, "FN") {
                        None => {}
                        Some(name) => match contact_prop(&mut vcard, "BDAY") {
                            None => {
                                urls_by_name.insert(name, contact_prop(&mut vcard, "URL"));
                            }
                            Some(birthday) => match yyyymmdd.captures(birthday.as_str()) {
                                None => {
                                    eprintln!("{} is not like {}", birthday, yyyymmdd);
                                    return empty_response(502);
                                }
                                Some(cap) => {
                                    urls_by_mdy_name.insert(
                                        (
                                            parse::<u8>(&cap, 2).unwrap(),
                                            parse::<u8>(&cap, 3).unwrap(),
                                            parse::<i16>(&cap, 1).unwrap_or(-1),
                                            name,
                                        ),
                                        contact_prop(&mut vcard, "URL"),
                                    );
                                }
                            },
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
                                        td.text(if year < 0 {
                                            format!("????-{}-{}", month, day)
                                        } else {
                                            format!("{}-{}-{}", year, month, day)
                                        })
                                    })
                                    .table_cell(|td| name_cell(td, name, url, &srch))
                                });
                            }

                            for (name, url) in urls_by_name {
                                table.table_row(|tr| {
                                    tr.table_cell(|td| td.text("?"))
                                        .table_cell(|td| name_cell(td, name, url, &srch))
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

fn parse<'a, I>(cap: &Captures<'a>, i: usize) -> Result<I, I::Err>
where
    I: FromStr,
{
    cap.get(i).unwrap().as_str().parse::<I>()
}

fn name_cell<'a>(
    td: &'a mut TableCellBuilder,
    name: String,
    url: Option<String>,
    srch: &Option<String>,
) -> &'a mut TableCellBuilder {
    match url {
        Some(url) => td.anchor(|a| a.target("_blank").href(url).text(name)),
        None => match &srch {
            Some(url) => td.anchor(|a| {
                a.target("_blank")
                    .href(format!(
                        "{}{}",
                        url,
                        utf8_percent_encode(name.as_str(), NON_ALPHANUMERIC)
                    ))
                    .text(name)
            }),
            None => td.text(name),
        },
    }
}
