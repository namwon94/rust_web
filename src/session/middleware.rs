use actix_web::error::InternalError;
use actix_web_lab::middleware::Next;
use actix_web::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::FromRequest;
use crate::session_state::TypedSession;
use crate::error::{e500, see_other};
use std::ops::Deref;
use actix_web::HttpMessage;

/*
String 타입은 std::marker::Copy 트레이트를 구현하지 않습니다. 
이는 Copy 트레이트가 "비트 단순 복사"를 의미하는데, String은 힙에 데이터를 저장하는 동적 크기 타입이어서 단순히 메모리 비트 사본을 만들어 복사할 경우, 
힙 메모리의 중복 해제로 인해 문제가 발생할 수 있기 때문입니다
*/
#[derive(Clone, Debug)]
pub struct EmailInfo(String);

impl std::fmt::Display for EmailInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Deref for EmailInfo {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub async fn reject_anonymous_users(
    mut req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, actix_web::Error> {
    let session = {
        let (http_request, payload) = req.parts_mut();
        TypedSession::from_request(&http_request, payload).await
    }?;

    match session.get_email().map_err(e500)? {
        Some(email) => {
            req.extensions_mut().insert(EmailInfo(email));
            next.call(req).await
        },
        None => {
            let response = see_other("/home");
            let e = anyhow::anyhow!("The user has not logged in");
            Err(InternalError::from_response(e, response).into())
        }
    }
}