use actix_web::{
    HttpRequest, cookie::{
        time, Cookie
    },
};
use chrono::{Utc, Duration};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode, errors::ErrorKind};
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
use crate::error::{
    JwtError,
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
    ) -> Result<String, JwtError> {
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
        let token = encode(
            &Header::default(), 
            &claims, 
            &EncodingKey::from_secret(self.secret.as_ref())
        )
        .map_err(|e| JwtError::Other(e.to_string()))?;

        Ok(token)
    }

    //refresh token 생성 함수
    pub fn create_refresh_token(
        &self,
        email: &str,
    ) -> Result<String, JwtError> {
        let jti = Uuid::new_v4().to_string();
        let expiration = Utc::now()
            .checked_add_signed(Duration::days(15))
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
        )
        .map_err(|e| JwtError::Other(e.to_string()))?;
        /*
        Redis에 Refresh Token 정보 저장
        Key : refresh_token:{email}:{jti}
        Value : token
        TTL : 7일
         */
        let mut con = self.redis_client.get_connection().map_err(|e| JwtError::RedisError(e.to_string()))?;
        //println!("jti(create) : {}", jti);
        let redis_key = format!("refresh_token:{}:{}", email, jti);
        con.set_ex::<_, _, ()>(&redis_key, &token, 7*24*60*60).map_err(|e| JwtError::RedisError(e.to_string()))?;
        /*
        반환 타입을 명시해야된다. -> Rust2024에서는 ()fallback을 금지한다.
         */

        Ok(token)
    }

    //access token 검증 함수
    pub fn verify_access_token(
        &self,
        token: &str
    ) -> Result<AccessTokenClaims, JwtError> {
        let token_data = decode::<AccessTokenClaims>(
            token,
            &DecodingKey::from_secret(self.secret.as_ref()),
            &Validation::default()
        )
        .map_err(|e| match *e.kind() {
            ErrorKind::ExpiredSignature => JwtError::ExpiredToken,
            ErrorKind::InvalidSignature => JwtError::InvalidSignature,
            ErrorKind::InvalidIssuer => JwtError::InvalidIssuer,
            ErrorKind::InvalidToken => JwtError::InvalidToken,
            _ => JwtError::Other(e.to_string()),
        })?;
        let claims = token_data.claims;

        Ok(claims)
    }

    //refresh token 검증 함수
    pub fn verify_refresh_token(
        &self,
        token: &str
    ) -> Result<RefreshTokenClaims, JwtError> {
        let token_data = decode::<RefreshTokenClaims> (
            token,
            &DecodingKey::from_secret(self.secret.as_ref()),
            &Validation::default()
        )
        .map_err(|e| match *e.kind() {
            ErrorKind::ExpiredSignature => JwtError::ExpiredToken,
            ErrorKind::InvalidSignature => JwtError::InvalidSignature,
            ErrorKind::InvalidIssuer => JwtError::InvalidIssuer,
            ErrorKind::InvalidToken => JwtError::InvalidToken,
            _ => JwtError::Other(e.to_string()),
        })?;

        let claims = token_data.claims;
        println!("claims.email : {}, claims.jti : {}", claims.email, claims.jti);
        let mut con = self.redis_client.get_connection().map_err(|e| JwtError::RedisError(e.to_string()))?;
        let redis_key = format!("refresh_token:{}:{}", claims.email, claims.jti);
        println!("redis_key : {}", redis_key);
        let exists: bool = con.exists(&redis_key).map_err(|e| JwtError::RedisError(e.to_string()))?;

        if !exists {
            return Err(JwtError::TokenRevoked)
        }

        Ok(claims)
    }

    //refresh_token rotate 함수
    pub fn rotate_refresh_token(
        &self,
        token: &str,
    ) -> Result<String, JwtError> {
        let token_data = decode::<RefreshTokenClaims>(
            &token,
            &DecodingKey::from_secret(self.secret.as_ref()),
            &Validation::default()
        )
        .map_err(|e| match *e.kind() {
            ErrorKind::ExpiredSignature => JwtError::ExpiredToken,
            ErrorKind::InvalidSignature => JwtError::InvalidSignature,
            ErrorKind::InvalidIssuer => JwtError::InvalidIssuer,
            ErrorKind::InvalidToken => JwtError::InvalidToken,
            _ => JwtError::Other(e.to_string()),
        })?;
        let claims = token_data.claims;
        self.remove_refresh_token(&token).map_err(|e| JwtError::Other(e.to_string()))?;
        
        let refresh_token = self.create_refresh_token(&claims.email).expect("Fail to create refresh token");

        Ok(refresh_token)
    }

    //refresh token으로 새로운 Access Token 발급
    pub fn refresh_access_token(
        &self,
        refresh_token: &str,
        role: Option<String>
    ) -> Result<String, JwtError> {
        let claims = self.verify_refresh_token(refresh_token)?;
        self.create_access_token(&claims.email, role).map_err(|e| e.into())
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
        token: &str,
    ) -> Result<(), JwtError> {
        let token_data = decode::<RefreshTokenClaims>(
            &token,
            &DecodingKey::from_secret(self.secret.as_ref()),
            &Validation::default()
        )
        .map_err(|e| match *e.kind() {
            ErrorKind::ExpiredSignature => JwtError::ExpiredToken,
            ErrorKind::InvalidSignature => JwtError::InvalidSignature,
            ErrorKind::InvalidIssuer => JwtError::InvalidIssuer,
            ErrorKind::InvalidToken => JwtError::InvalidToken,
            _ => JwtError::Other(e.to_string()),
        })?;
        let claims = token_data.claims;
        let mut con = self.redis_client.get_connection().map_err(|e| JwtError::RedisError(e.to_string()))?;
        //println!("jti(remove) : {}", claims.jti);
        let redis_key = format!("refresh_token:{}:{}", claims.email, claims.jti);
        //println!("redis key: {}", redis_key);
        con.del::<_, ()>(&redis_key).map_err(|e| JwtError::RedisError(e.to_string()))?;

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