extern crate actix_web;
extern crate futures;
#[macro_use]
extern crate log;
extern crate log4rs;
extern crate http;

use futures::{Future, future::ok};
use actix_web::{server, App, AsyncResponder, Responder, Error, client, HttpRequest, HttpResponse, HttpMessage, http::StatusCode};

fn main() {
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();
    server::new(|| {
            App::new().resource("/test", |r| r.f(test))
        })
        .keep_alive(75)
        .bind("0.0.0.0:8088")
        .unwrap()
        .run();
}

fn test(_req: &HttpRequest) -> impl Responder {
    let token = "this_is_not_a_proper_token";
    return client::get("https://www.googleapis.com/oauth2/v1/userinfo?alt=json")
        .header("Authorization", format!("Bearer {}", token))
//        .header("accept-encoding", "identity")
        .finish()
        .unwrap()
        .send()
        .from_err::<Error>()
        .and_then(move |info_response| {
            info!("did the thing, status: {}", info_response.status());
            return info_response.body().from_err();
        })
        .and_then(move |body| {
            info!("got the data");
            let mut response = HttpResponse::Ok();
            return ok::<_, Error>(response.body(format!("<html><body>OK!  <br>response:<br><pre>{}</pre></body></html>", String::from_utf8(body.to_vec()).unwrap())));
        })
        .or_else(move |err| {
            info!("boom! {}", err);
            let mut response = HttpResponse::Ok();
            return ok::<_, Error>(response.status(StatusCode::FORBIDDEN).body(format!("<html><body>bad/error in response: {}</body></html>", err)));
        })
        .responder();
}
