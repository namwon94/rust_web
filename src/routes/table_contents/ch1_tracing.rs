use actix_web::{HttpResponse, Result, http::header::ContentType,};
use askama::Template; 


#[derive(Template)]
#[template(path = "ch1_1/n1_basic.html")]
struct TracingBasicTemplate;

pub async fn tracing_basic() -> Result<HttpResponse> {
    let template = TracingBasicTemplate;
    let rendered = template.render().map_err(|e| {
        actix_web::error::ErrorInternalServerError(e)
    })?;
    Ok(HttpResponse::Ok().content_type(ContentType::html()).body(rendered))
}

#[derive(Template)]
#[template(path = "ch1_1/n2_settings.html")]
struct TracingSettingsTemplate;

pub async fn tracing_settings() -> Result<HttpResponse> {
    let template = TracingSettingsTemplate;
    let rendered = template.render().map_err(|e| {
        actix_web::error::ErrorInternalServerError(e)
    })?;
    Ok(HttpResponse::Ok().content_type(ContentType::html()).body(rendered))
}

#[derive(Template)]
#[template(path = "ch1_1/n3_span_event.html")]
struct TracingSpanEventTemplate;

pub async fn tracing_span_event() -> Result<HttpResponse> {
    let template = TracingSpanEventTemplate;
    let rendered = template.render().map_err(|e| {
        actix_web::error::ErrorInternalServerError(e)
    })?;
    Ok(HttpResponse::Ok().content_type(ContentType::html()).body(rendered))
}

#[derive(Template)]
#[template(path = "ch1_2/n1_basic.html")]
struct SubscriberBasicTemplate;

pub async fn subscirber_basic() -> Result<HttpResponse> {
    let template = SubscriberBasicTemplate;
    let rendered = template.render().map_err(|e| {
        actix_web::error::ErrorInternalServerError(e)
    })?;
    Ok(HttpResponse::Ok().content_type(ContentType::html()).body(rendered))
}

#[derive(Template)]
#[template(path = "ch1_2/n2_example.html")]
struct SubscriberExampleTemplate;

pub async fn subscirber_example() -> Result<HttpResponse> {
    let template = SubscriberExampleTemplate;
    let rendered = template.render().map_err(|e| {
        actix_web::error::ErrorInternalServerError(e)
    })?;
    Ok(HttpResponse::Ok().content_type(ContentType::html()).body(rendered))
}
