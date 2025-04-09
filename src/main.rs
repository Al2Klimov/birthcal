mod cli;

use cgi::{empty_response, handle, text_response, Request, Response};
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
            return empty_response(502);
        }
        Ok(mut resp) => {
            for i in ical::VcardParser::new(BufReader::new(resp.body_mut().as_reader())) {
                match i {
                    Err(err) => {
                        eprintln!("GET {}: {}", url, err);
                        return empty_response(502);
                    }
                    Ok(vcard) => {}
                }
            }
        }
    }

    text_response(501, "")
}
