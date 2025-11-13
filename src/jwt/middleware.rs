use actix_web::{
    /* 
    dev::{
        ServiceRequest, ServiceResponse
    }, 
    */
    HttpRequest,
    //error::ErrorUnauthorized, 
    //Error,
    //HttpMessage
};
//use actix_web_lab::middleware::Next;
use chrono::{Utc, Duration};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, encode, decode};
use serde::{
    Serialize,
    Deserialize
};

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessTokenClaims {
    //이메일
    pub email: String, 
    //만료 시간
    pub exp: usize,
    //발급 시간
    pub iat: usize,
    //사용자 역할
    pub role: Option<String>,
}

#[derive(Debug, Clone)]
pub struct JwtService {
    pub secret: String,
}
/*
&self : 서버 실행 시 이미 jwt_secret를 받기 때문에 해당 메서드를 불러올때 직접 넘길 필요 없다.
*/
impl JwtService {
    pub fn new(secret: String) -> Self {
        Self{secret}
    }
    //access token 생성 함수
    pub fn create_access_token(
        &self,
        email: &str,
        role: Option<String>,
    ) -> Result<String, jsonwebtoken::errors::Error> {
        let expiration = Utc::now()
            .checked_add_signed(Duration::minutes(15))
            .expect("valid timestamp")
            .timestamp() as usize;

        let claims = AccessTokenClaims {
            email: email.to_owned(),
            exp: expiration,
            iat: Utc::now().timestamp() as usize,
            role
        };
        //println!("sucess");
        encode(&Header::default(), &claims, &EncodingKey::from_secret(self.secret.as_ref()))
    }

    //토큰 추출 함수
    pub fn extract_access_token(
        &self, 
        req: &HttpRequest
    ) -> Option<String> {
        req.headers()
            .get("Authorization")?
            .to_str()
            .ok()?
            .strip_prefix("Bearer ")
            .map(|s| s.to_string())
    }

    //토큰 검증 함수
    pub fn verify_access_token(
        &self,
        token: &str
    ) -> Result<AccessTokenClaims, jsonwebtoken::errors::Error> {
        let token_data = decode::<AccessTokenClaims>(
            token,
            &DecodingKey::from_secret(self.secret.as_ref()),
            &Validation::default()
        )?;

        Ok(token_data.claims)
    }

}

/* 
//미들웨어에서 사용하는 jwt 인증
pub async fn jwt_auth_middleware(
    secret: JwtService,
    req: ServiceRequest,
    next: Next<impl actix_web::body::MessageBody>,
) -> Result<ServiceResponse<impl actix_web::body::MessageBody>, Error>  {
    //1. 토큰 추출
    let token = JwtService::extract_access_token(&req).ok_or_else(|| ErrorUnauthorized("Missing or invalid Authoriztion header"))?;

    //2. 토큰 검증
    let claims = JwtService::verify_access_token(&secret, &token)
        .map_err(|e| ErrorUnauthorized(format!("Invalid token: {}", e)))?;

    //3. 검증된 Claims를 request extensions에 저장
    req.extensions_mut().insert(claims);

    //4. 다음 미들웨어/핸들러로 전달
    next.call(req).await
}
*/