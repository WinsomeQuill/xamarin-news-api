pub mod user {
    use std::io::Write;
    use crate::postgresql::models::model_user::user::{
        RegisterUser,
    };
    use crate::services::{
        get_query_param,
        json_error,
        json_success,
        read_body_bytes
    };
    use actix_web::{
        get,
        post,
        web,
        HttpRequest,
        Responder,
        HttpResponse,
    };
    use base64::Engine;
    use base64::engine::general_purpose;
    use crate::postgresql::postgresql_manager::Connect;
    use crate::logger::log::{Level, log};
    use serde_json::{json, Value};

    #[post("/insert-user")]
    pub async fn insert_user(conn: web::Data<Connect>, mut payload: web::Payload) -> impl Responder {
        let body = match read_body_bytes(&mut payload).await {
            Ok(o) => o,
            Err(_) => return HttpResponse::Ok().json(
                json_error("Request overflow!")
            )
        };

        let user = match serde_json::from_slice::<RegisterUser>(&body) {
            Ok(o) => o,
            Err(e) => {
                log(Level::Error, "[POST][insert-user] >>> serde_json::from_slice::<RegisterUser>",
                    &format!("Handle: {}", e)
                );

                return HttpResponse::Ok().json(
                    json_error("Error request!")
                )
            },
        };

        if let Ok(o) = conn.exist_user_by_login(&user.login).await {
            if o {
                return HttpResponse::Ok().json(
                    json_error("Login already registered!")
                );
            }
        }

        if let Err(e) = conn.insert_user(&user).await {
            log(Level::Error, "[POST][insert-user] >>> conn.insert_user(&user)",
                &format!("Handle: {}", e)
            );

            return HttpResponse::Ok().json(
                json_error("Error")
            );
        };

        HttpResponse::Ok().json(
            json_success("Success")
        )
    }

    #[get("/login-user")]
    pub async fn login_user(conn: web::Data<Connect>, req: HttpRequest) -> impl Responder {
        let login = match get_query_param::<String>(&req, "login").await {
            Ok(o) => o,
            Err(e) => return HttpResponse::BadRequest().json(
                json_error(e)
            )
        };

        let password = match get_query_param::<String>(&req, "password").await {
            Ok(o) => o,
            Err(e) => return HttpResponse::BadRequest().json(
                json_error(e)
            )
        };

        let user = match conn.get_user_by_login_and_pass(&login, &password).await {
            Ok(o) => o,
            Err(e) => {
                log(Level::Error, "[GET][login-user] >>> conn.get_user_by_login_and_pass",
                    &format!("Handle: {}", e)
                );

                return HttpResponse::Ok().json(
                    json_error("User not found!")
                );
            },
        };

        HttpResponse::Ok().json(
            json_success(user)
        )
    }

    #[get("/user-info")]
    pub async fn user_info(conn: web::Data<Connect>, req: HttpRequest) -> impl Responder {
        let id = match get_query_param::<i32>(&req, "user_id").await {
            Ok(o) => o,
            Err(e) => return HttpResponse::BadRequest().json(
                json_error(e)
            )
        };

        let user = match conn.get_user_info_by_id(id).await {
            Ok(o) => o,
            Err(e) => {
                log(Level::Error, "[GET][user-info] >>> conn.get_user_info_by_id",
                    &format!("Handle: {}", e)
                );

                return HttpResponse::Ok().json(
                    json_error("User not found!")
                );
            },
        };

        HttpResponse::Ok().json(
            json_success(user)
        )
    }

    #[get("/user-count-followers")]
    pub async fn user_count_followers(conn: web::Data<Connect>, req: HttpRequest) -> impl Responder {
        let id = match get_query_param::<i32>(&req, "user_id").await {
            Ok(o) => o,
            Err(e) => return HttpResponse::Ok().json(
                json_error(e)
            )
        };

        let count = match conn.get_user_count_followers(id).await {
            Ok(o) => o,
            Err(e) => {
                log(Level::Error, "[GET][user-count-followers] >>> conn.get_user_count_followers",
                    &format!("Handle: {}", e)
                );

                return HttpResponse::Ok().json(
                    json_error(0)
                );
            },
        };

        HttpResponse::Ok().json(
            json_success(count)
        )
    }

    #[get("/is-user-followed")]
    pub async fn is_user_followed(conn: web::Data<Connect>, req: HttpRequest) -> impl Responder {
        let author_user_id = match get_query_param::<i32>(&req, "author_user_id").await {
            Ok(o) => o,
            Err(e) => return HttpResponse::BadRequest().json(
                json_error(e)
            )
        };

        let follower_user_id = match get_query_param::<i32>(&req, "follower_user_id").await {
            Ok(o) => o,
            Err(e) => return HttpResponse::BadRequest().json(
                json_error(e)
            )
        };

        let result = match conn.is_user_followed_to_user(author_user_id, follower_user_id).await {
            Ok(o) => o,
            Err(e) => {
                log(Level::Error, "[GET][is-user-followed] >>> conn.is_user_followed_to_user",
                    &format!("Handle: {}", e)
                );

                return HttpResponse::Ok().json(
                    json_error(false)
                );
            },
        };

        HttpResponse::Ok().json(
            json_success(result)
        )
    }

    #[deprecated(since = "0.1.2", note = "Не используется так-как есть запрос user-info")]
    #[get("/get-profile-avatar")]
    pub async fn get_profile_avatar(conn: web::Data<Connect>, req: HttpRequest) -> impl Responder {
        let login = match get_query_param::<String>(&req, "login").await {
            Ok(o) => o,
            Err(e) => return HttpResponse::BadRequest().json(
                json_error(e)
            )
        };

        let (crop_avatar, full_avatar) = match conn.get_avatar_by_login(&login).await {
            Ok(o) => o,
            Err(sqlx::Error::RowNotFound) => return HttpResponse::Ok().json(
                json_success("User not found!")
            ),
            Err(e) => {
                log(Level::Error, "[GET][get-profile-avatar] >>> conn.get_avatar_by_login",
                    &format!("Handle: {}", e)
                );

                return HttpResponse::Ok().json(
                    json_success("Error request!")
                );
            },
        };

        let result = json!({
            "crop_avatar": crop_avatar,
            "full_avatar": full_avatar,
        });

        HttpResponse::Ok().json(
            json_success(result)
        )
    }

    #[post("/set-profile-avatar")]
    pub async fn set_profile_avatar(conn: web::Data<Connect>, mut payload: web::Payload) -> impl Responder {
        let body = match read_body_bytes(&mut payload).await {
            Ok(o) => o,
            Err(_) => return HttpResponse::Ok().json(
                json_error("Request overflow!")
            )
        };

        let value = match serde_json::from_slice::<Value>(&body) {
            Ok(o) => o,
            Err(e) => {
                log(Level::Error, "[POST][set-profile-avatar] >>> serde_json::from_slice::<Value>",
                    &format!("Handle: {}", e)
                );
                
                return HttpResponse::Ok().json(
                    json_error("Error request!")
                )
            },
        };

        let crop_avatar = general_purpose::STANDARD.decode(value["crop_avatar"].as_str().unwrap()).unwrap();
        let full_avatar = general_purpose::STANDARD.decode(value["full_avatar"].as_str().unwrap()).unwrap();
        let login = value["login"].as_str().unwrap();

        match conn.set_avatar_by_login(login, &crop_avatar, &full_avatar).await {
            Err(sqlx::Error::RowNotFound) | Ok(_) => {
                HttpResponse::Ok().json(
                    json_success("Success")
                );
            },
            Err(e) => {
                log(Level::Error, "[POST][set-profile-avatar] >>> conn.set_avatar_by_login",
                    &format!("Handle: {}", e)
                );
                
                return HttpResponse::Ok().json(
                    json_error("Error request!")
                )
            },
        };

        HttpResponse::Ok().json(
            json_success("Success")
        )
    }

    #[post("/following-user")]
    pub async fn following_user(conn: web::Data<Connect>, mut payload: web::Payload) -> impl Responder {
        let body = match read_body_bytes(&mut payload).await {
            Ok(o) => o,
            Err(_) => return HttpResponse::Ok().json(
                json_error("Request overflow!")
            )
        };

        let value = match serde_json::from_slice::<Value>(&body) {
            Ok(o) => o,
            Err(e) => {
                log(Level::Error, "[POST][following-user] >>> serde_json::from_slice::<Value>",
                    &format!("Handle: {}", e)
                );

                return HttpResponse::Ok().json(
                    json_error("Error request!")
                )
            },
        };

        let author_id = value["author_id"].as_i64().unwrap() as i32;
        let follower_id = value["follower_id"].as_i64().unwrap() as i32;

        let result = match conn.is_user_followed_to_user(author_id, follower_id).await {
            Ok(o) => o,
            Err(e) => {
                log(Level::Error, "[POST][following-user] >>> conn.is_user_followed_to_user",
                    &format!("Handle: {}", e)
                );

                return HttpResponse::Ok().json(
                    json_error(false)
                );
            },
        };

        if result {
            return HttpResponse::Ok().json(
                json_error("You already subscribed this author!")
            );
        }

        match conn.set_following_user(author_id, follower_id).await {
            Ok(_) | Err(sqlx::Error::RowNotFound) => {
                HttpResponse::Ok().json(
                    json_success("Success")
                );
            },
            Err(e) => {
                log(Level::Error, "[POST][following-user] >>> conn.set_following_user",
                    &format!("Handle: {}", e)
                );

                return HttpResponse::Ok().json(
                    json_error("Error request!")
                )
            },
        };

        HttpResponse::Ok().json(
            json_success("Success")
        )
    }

    #[post("/remove-following-user")]
    pub async fn remove_following_user(conn: web::Data<Connect>, mut payload: web::Payload) -> impl Responder {
        let body = match read_body_bytes(&mut payload).await {
            Ok(o) => o,
            Err(_) => return HttpResponse::Ok().json(
                json_error("Request overflow!")
            )
        };

        let value = match serde_json::from_slice::<Value>(&body) {
            Ok(o) => o,
            Err(e) => {
                log(Level::Error, "[POST][remove-following-user] >>> serde_json::from_slice::<Value>",
                    &format!("Handle: {}", e)
                );

                return HttpResponse::Ok().json(
                    json_error("Error request!")
                )
            },
        };

        let author_id = value["author_id"].as_i64().unwrap() as i32;
        let follower_id = value["follower_id"].as_i64().unwrap() as i32;

        let result = match conn.is_user_followed_to_user(author_id, follower_id).await {
            Ok(o) => o,
            Err(e) => {
                log(Level::Error, "[POST][remove-following-user] >>> conn.is_user_followed_to_user",
                    &format!("Handle: {}", e)
                );

                return HttpResponse::Ok().json(
                    json_error(false)
                );
            },
        };

        if !result {
            return HttpResponse::Ok().json(
                json_error("You are not subscribed this author!")
            );
        }

        match conn.remove_following_user(author_id, follower_id).await {
            Ok(_) | Err(sqlx::Error::RowNotFound) => {
                HttpResponse::Ok().json(
                    json_success("Success")
                );
            },
            Err(e) => {
                log(Level::Error, "[POST][following-user] >>> conn.set_following_user",
                    &format!("Handle: {}", e)
                );

                return HttpResponse::Ok().json(
                    json_error("Error request!")
                )
            },
        };

        HttpResponse::Ok().json(
            json_success("Success")
        )
    }
}