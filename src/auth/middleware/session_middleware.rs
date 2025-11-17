use actix_web::error::InternalError;
use actix_web_lab::middleware::Next;
use actix_web::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::FromRequest;
use std::ops::Deref;
use actix_web::HttpMessage;
use crate::error::{e500, see_other};
use crate::auth::TypedSession;

/*
String 타입은 std::marker::Copy 트레이트를 구현하지 않습니다. 
이는 Copy 트레이트가 "비트 단순 복사"를 의미하는데, String은 힙에 데이터를 저장하는 동적 크기 타입이어서 단순히 메모리 비트 사본을 만들어 복사할 경우, 
힙 메모리의 중복 해제로 인해 문제가 발생할 수 있기 때문입니다.
### String의 메모리 구조 - (스택 : 포인터, 길이, 용랭(24바이트) / 힙 : 실제 문자열 데이터) 
    -> 비트 복사를 하면 포인터가 복사되어 두 개의 String이 같음 힙 메모리를 가리키게 된다. 둘 다 소멸될때 같은 메모리를 두 번 해제하려고 한다.(double free)
### Uuid의 메모리 구조 - (스택: UUID의 모든 데이터 (16바이트) / 힙: 사용하지 않음
    -> 비트 복사를 해도 16바이트의 데이터가 그대로 복사될 뿐, 포인터가 없으므로 중복 해제 문제가 발생하지 않습니다.
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

