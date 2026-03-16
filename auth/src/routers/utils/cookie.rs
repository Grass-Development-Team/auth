use axum_extra::extract::{
    CookieJar,
    cookie::{Cookie, SameSite},
};

use crate::internal::session::SESSION_TTL_SECONDS;

pub trait CookieJarExt {
    fn remove_session_cookie(self) -> Self;
}

pub fn build_session_cookie(session: String, secure: bool) -> Cookie<'static> {
    Cookie::build(("session", session))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(secure)
        .max_age(time::Duration::seconds(SESSION_TTL_SECONDS as i64))
        .build()
}

impl CookieJarExt for CookieJar {
    fn remove_session_cookie(self) -> Self {
        let mut cookie = Cookie::new("session", "");
        cookie.set_path("/");
        self.remove(cookie)
    }
}
