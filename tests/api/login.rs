use crate::helpers::{spawn_app, assert_is_redirect, assert_is_message};

#[tokio::test]
async fn login_try_wrong_data() {
    //Arrange
    let app = spawn_app().await;

    //Act
    let login_body = serde_json::json!({
        "email": "test@example.com",
        "password": "random-password"
    });
    let response = app.post_login_json(&login_body).await;
    //println!("Status: {}", response.status());
    //let body = response.text().await.unwrap();
    //println!("Response body: {}", body);

    //panic!("Check the ouput above");
    
    assert_is_redirect(response, 401, "/home").await;
}

#[tokio::test]
async fn an_register_same_email() {
    //Arrange
    let app = spawn_app().await;

    //Act1 - 테스트 email, password 가져오기
    let register_body = serde_json::json!({
        "email": app.test_user.email,
        "password": "random_password",
        "name": "random_name",
        "nickname": "random_nickname",
    });
    let response = app.post_register(&register_body).await;

    assert_is_message(response, 400).await;
    
}
