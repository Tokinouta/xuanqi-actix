use actix_session::Session;
use actix_web::{
    post, web, HttpResponse, Result,
};
use mongodb::Client;
use serde::{Deserialize, Serialize};

use crate::model::{
    database::{add_user, verify_user},
    User,
};

#[post("/view")]
async fn login(
    client: web::Data<Client>,
    user_id: web::Form<User>,
    session: Session,
) -> Result<HttpResponse> {
    let user = user_id.into_inner();
    let id = match verify_user(&client, &user).await {
        Ok(r) => {
            if r {
                user.name
            } else {
                return Ok(HttpResponse::InternalServerError().body("no such user"));
            }
        }
        Err(_) => return Ok(HttpResponse::InternalServerError().body("database error")),
    };
    session.insert("user_id", &id)?;
    session.renew();

    let counter: i32 = session
        .get::<i32>("counter")
        .unwrap_or(Some(0))
        .unwrap_or(0);

    Ok(HttpResponse::Ok().json(IndexResponse {
        user_id: Some(id),
        counter,
    }))
}

#[post("/add-item")]
pub fn add_item(
    client: web::Data<Client>,
    user_id: web::Form<User>,
    session: Session,
) -> Result<HttpResponse>{
    // 没有对应的session会返回一个空对象，判断里面是不是空的可以判定登录
    let user_id: Option<String> = session.get::<String>("user_id").unwrap();
    let counter: i32 = session
        .get::<i32>("counter")
        .unwrap_or(Some(0))
        .map_or(1, |inner| inner + 1);
    session.insert("counter", counter)?;

    Ok(HttpResponse::Ok().json(IndexResponse { user_id, counter }))
}