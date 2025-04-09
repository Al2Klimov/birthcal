mod cli;

use cgi::{empty_response, handle, text_response, Request, Response};

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
        Ok(mut resp) => {}
    }

    text_response(501, "")
}
