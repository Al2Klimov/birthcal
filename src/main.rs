use cgi::{handle, text_response, Request, Response};

fn main() {
    handle(handler)
}

fn handler(_: Request) -> Response {
    text_response(501, "")
}
