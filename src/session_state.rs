use actix_session::{Session, SessionExt};
use actix_web::dev::Payload;
use actix_web::{FromRequest, HttpRequest};
use std::future::{Ready, ready};

pub struct TypedSession(Session);

impl TypedSession{
    const EMAIL_KEY: &'static str = "email";

    pub fn renew(&self) {
        self.0.renew()
    }

    pub fn insert_email(&self, email: String) -> Result<(), serde_json::Error> {
        self.0.insert(Self::EMAIL_KEY, email)
    }

    pub fn get_email(&self) -> Result<Option<String>, serde_json::Error> {
        self.0.get(Self::EMAIL_KEY)
    }

    pub fn log_out(self) {
        self.0.purge()
    }
}

impl FromRequest for TypedSession {
    type Error = <Session as FromRequest>::Error;
    type Future = Ready<Result<TypedSession, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        ready(Ok(TypedSession(req.get_session())))
    }
}