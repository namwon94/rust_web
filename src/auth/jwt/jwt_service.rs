use actix_web::{
    HttpRequest,
};
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

    pub fn extract_access_token(
        &self,
        req: &HttpRequest
    ) -> Option<String> {
        req.cookie("access_token").map(|s| s.value().to_string())
    }

    /* 
    //토큰 추출 함수(Api용)
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
    */

}