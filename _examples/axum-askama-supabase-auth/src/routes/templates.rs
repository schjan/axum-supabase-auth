use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
pub struct Index;

#[derive(Template)]
#[template(path = "login.html")]
pub struct Login<'a> {
    pub next: Option<&'a str>,
}

#[derive(Template)]
#[template(path = "register.html")]
pub struct Register;

#[derive(Template)]
#[template(path = "profile.html")]
pub struct Profile;
