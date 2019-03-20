#[macro_use]
extern crate serde_derive;
extern crate serde_qs as qs;
extern crate serde;

#[macro_use]
extern crate rouille;

use rouille::Response;
use std::io;
use std::process::Command;
use serde::{Serialize, Deserialize};

#[derive(PartialEq, Serialize, Deserialize)]
struct Params {
    next: String
}

const CREDENTIALS: [(&'static str, &'static str); 2] = [("admin", "password"), ("teddy", "bear")];

fn main() {
    let web_content = include_str!("index.html");
    rouille::start_server("0.0.0.0:4280", move |request| {
        rouille::log(request, io::stdout(), || {
            let params: Option<Params> = qs::from_str(request.raw_query_string()).ok();

            if let Ok(input) = post_input!(request, { username: String, password: String }) {
                if !CREDENTIALS.contains(&(input.username.as_str(), input.password.as_str())) {
                    return Response::text("Invalid credentials").with_status_code(401);
                }

                let res = Command::new("pfctl")
                    .args(&[
                        "-t",
                        "whitelist",
                        "-T",
                        "add",
                        &request.remote_addr().ip().to_string(),
                    ])
                    .spawn()
                    .and_then(|mut c| c.wait());

                if let Err(_) = res {
                    return Response::text("internal server error").with_status_code(500);
                }

                let resp = Response::text(format!(
                    "Hello {}! You are now logged in and can browse the web!",
                    input.username
                ));

                if let Some(params) = params {
                    return resp.with_additional_header("Location", params.next)
                        .with_status_code(302);
                }

                return resp;
            }

            if params == None {
                let next = format!("http://{}{}", request.header("Host").unwrap_or("localhost"), request.raw_url());
                let query = qs::to_string(&Params { next: String::from(next) });
                return match query {
                    Ok(q) => Response::text("Redirecting")
                        .with_additional_header("Location", format!("http://192.168.10.1/?{}", q))
                        .with_status_code(302),
                    Err(_) => Response::text("internal server error").with_status_code(500)
                }
            }

            Response::html(web_content)
        })
    });
}
