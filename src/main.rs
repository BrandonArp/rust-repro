extern crate actix_web;
extern crate futures;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate log;
extern crate log4rs;

use futures::{Future, future::ok};
use actix_web::{server, App, AsyncResponder, Responder, Error, client, HttpRequest, HttpResponse, HttpMessage, http::StatusCode};
use std::time::Duration;

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

#[derive(Deserialize)]
struct GoogleUserInfo {
    email: String,
}

fn test(_req: &HttpRequest) -> impl Responder {
    let token = "this_is_not_a_proper_token";
    return client::get("https://www.googleapis.com/oauth2/v1/userinfo?alt=json")
        .header("Authorization", format!("Bearer {}", token))
        .timeout(Duration::from_secs(60))
        .finish()
        .unwrap()
        .send()
        .conn_timeout(Duration::from_secs(5))
        .from_err::<Error>()
        .and_then(move |info_response| {
            info!("did the thing, status: {}", info_response.status());
            let body = info_response.body().from_err();
            let result = body.map(|body| {
                info!("building the json object");
                let user_info: GoogleUserInfo = serde_json::from_slice(&body).unwrap();
                return user_info;
            });
            return result
        })
        .and_then(move |info| {
            info!("got the userinfo: {}", info.email);
            let mut response = HttpResponse::Ok();
            return ok::<_, Error>(response.body("OK!"));
        })
        .or_else(move |err| {
            info!("boom! {}", err);
            let mut response = HttpResponse::Ok();
            return ok::<_, Error>(response.status(StatusCode::FORBIDDEN).body(format!("bad/incorrect oauth2 code in response: {}", err)));
        })
        .responder();
}
