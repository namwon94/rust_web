use actix_web::{
    dev::{
        ServiceRequest, ServiceResponse
    }, 
    error::ErrorUnauthorized, 
    Error,
    HttpMessage
};
use actix_web_lab::middleware::Next;
use crate::auth::JwtService;


//미들웨어에서 사용하는 jwt 인증
pub async fn jwt_auth_middleware(
    jwt_service: JwtService,
    req: ServiceRequest,
    next: Next<impl actix_web::body::MessageBody>,
) -> Result<ServiceResponse<impl actix_web::body::MessageBody>, Error>  {
    //1. HttpRequest 추출
    let http_req = req.request();
    //2. 토큰 추출
    let token = jwt_service.extract_access_token(&http_req).ok_or_else(|| ErrorUnauthorized("Missing or invalid Authoriztion header"))?;

    //3. 토큰 검증
    let claims = jwt_service.verify_access_token(&token)
        .map_err(|e| ErrorUnauthorized(format!("Invalid token: {}", e)))?;

    //4. 검증된 Claims를 request extensions에 저장
    http_req.extensions_mut().insert(claims);

    //5. 다음 미들웨어/핸들러로 전달
    next.call(req).await
}

