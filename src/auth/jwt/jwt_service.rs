use actix_web::{
    HttpRequest, cookie::{
        time, Cookie
    },
};
use chrono::{Utc, Duration};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, encode, decode};
use serde::{
    Serialize,
    Deserialize
};
use redis::{
    Client,
    Commands, 
    //RedisResult,
};
use uuid::Uuid;

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

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshTokenClaims {
    //이메일
    pub email: String,
    //만료 시간
    pub exp: usize,
    //발급시간
    pub iat: usize,
    //JWT ID(고유 식별자)
    pub jti: String,
}

#[derive(Debug, Clone)]
pub struct JwtService {
    pub secret: String,
    pub redis_client: Client,
}
/*
&self : 서버 실행 시 이미 jwt_secret를 받기 때문에 해당 메서드를 불러올때 직접 넘길 필요 없다.
*/
impl JwtService {
    pub fn new(secret: String, redis_client: Client) -> Self {
        Self{secret, redis_client}
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

    //refresh token 생성 함수
    pub fn create_refresh_token(
        &self,
        email: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let jti = Uuid::new_v4().to_string();
        let expiration = Utc::now()
            .checked_add_signed(Duration::minutes(15))
            .expect("valid timestamp")
            .timestamp() as usize;
        let claims = RefreshTokenClaims {
            email: email.to_owned(),
            exp: expiration,
            iat: Utc::now().timestamp() as usize,
            jti: jti.clone(),
        };
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_ref())
        )?;
        /*
        Redis에 Refresh Token 정보 저장
        Key : refresh_token:{email}:{jti}
        Value : token
        TTL : 7일
         */
        let mut con = self.redis_client.get_connection()?;
        println!("jti(create) : {}", jti);
        let redis_key = format!("refresh_token:{}:{}", email, jti);
        con.set_ex::<_, _, ()>(&redis_key, &token, 7*24*60*60)?;
        /*
        반환 타입을 명시해야된다. -> Rust2024에서는 ()fallback을 금지한다.
         */

        Ok(token)
    }

    //access token 검증 함수
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

    //access token 추출 함수(쿠키용)
    pub fn extract_access_token(
        &self,
        req: &HttpRequest
    ) -> Option<String> {
        req.cookie("access_token").map(|s| s.value().to_string())
    }

    //refresh token 추출 함수
    pub fn extract_refresh_token(
        &self,
        req: &HttpRequest
    ) -> Option<String> {
        req.cookie("refresh_token").map(|s| s.value().to_string())
    }

    //refresh token 삭제(Redis) 함수
    pub fn remove_refresh_token(
        &self,
        req: &HttpRequest,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let token = self.extract_refresh_token(req).ok_or("Refresh token not found in cookie")?;
        let token_data = decode::<RefreshTokenClaims>(
            &token,
            &DecodingKey::from_secret(self.secret.as_ref()),
            &Validation::default()
        )?;
        let claims = token_data.claims;
        let mut con = self.redis_client.get_connection()?;
        //println!("jti(remove) : {}", claims.jti);
        let redis_key = format!("refresh_token:{}:{}", claims.email, claims.jti);
        //println!("redis key: {}", redis_key);
        con.del::<_, ()>(&redis_key)?;

        Ok(())
    }

    //access_token, refresh_token 쿠키삭제
    pub fn remove_token_cookie(
        &self,
        name: &str,
    ) -> Cookie<'static> {
        Cookie::build(name.to_owned(), "")
            .path("/")
            .http_only(true)
            .max_age(time::Duration::ZERO)
            .finish()
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