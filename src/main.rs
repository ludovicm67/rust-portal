#[macro_use]
extern crate rouille;

use rouille::Response;
use std::io;
use std::process::Command;

const CREDENTIALS: [(&'static str, &'static str); 2] = [("admin", "password"), ("teddy", "bear")];

fn main() {
    let web_content = include_str!("index.html");
    rouille::start_server("0.0.0.0:4280", move |request| {
        rouille::log(request, io::stdout(), || {
            if let Ok(input) = post_input!(request, { username: String, password: String }) {
                if !CREDENTIALS.contains(&(input.username.as_str(), input.password.as_str())) {
                    return Response::text("Invalid credentials").with_status_code(401);
                }
                Command::new("pfctl")
                    .args(&[
                        "-t",
                        "whitelist",
                        "-T",
                        "add",
                        &request.remote_addr().ip().to_string(),
                    ])
                    .spawn();
                let resp = Response::text(format!(
                    "Hello {}! You are now logged in and can browse the web!",
                    input.username
                ));
                let next = request.get_param("next");
                if let Some(next) = next {
                    resp.with_additional_header("Location", next)
                        .with_status_code(301)
                } else {
                    resp
                }
            } else {
                Response::html(web_content)
            }
        })
    });
}
