pub mod article {
    use crate::services::{
        get_query_param,
        json_error,
        json_success,
        read_body_bytes,
    };
    use actix_web::{
        get,
        post,
        web,
        HttpRequest,
        Responder,
        HttpResponse,
    };
    use crate::postgresql::postgresql_manager::Connect;
    use crate::logger::log::{Level, log};
    use serde_json::Value;
    use crate::postgresql::models::model_article::article::{
        InsertArticle,
        InsertComment,
        InsertReaction,
    };

    #[post("/insert-article")]
    pub async fn insert_article(conn: web::Data<Connect>, mut payload: web::Payload) -> impl Responder {
        let body = match read_body_bytes(&mut payload).await {
            Ok(o) => o,
            Err(_) => return HttpResponse::Ok().json(
                json_error("Request overflow!")
            )
        };

        let article = match serde_json::from_slice::<InsertArticle>(&body) {
            Ok(o) => o,
            Err(e) => {
                log(Level::Error, "[POST][insert-article] >>> serde_json::from_slice::<CropArticle>",
                    &format!("Handle: {}", e)
                );

                return HttpResponse::Ok().json(
                    json_error("Error request!")
                )
            },
        };

        if let Err(e) = conn.insert_article(&article).await {
            log(Level::Error, "[POST][insert-article] >>> conn.insert_article(&article)",
                &format!("Handle: {}", e)
            );

            return HttpResponse::Ok().json(
                json_error("Error")
            );
        }

        HttpResponse::Ok().json(
            json_success("Success")
        )
    }

    #[get("/get-articles")]
    pub async fn get_articles(conn: web::Data<Connect>, req: HttpRequest) -> impl Responder {
        let user_id = match get_query_param::<i32>(&req, "user_id").await {
            Ok(o) => o,
            Err(e) => return HttpResponse::BadRequest().json(
                json_error(e)
            )
        };

        let articles = match conn.get_articles(user_id).await {
            Ok(o) => o,
            Err(e) => {
                log(Level::Error, "[GET][get-articles] >>> conn.get_articles",
                    &format!("Handle: {}", e)
                );

                return HttpResponse::Ok().json(
                    json_error("Error!")
                );
            },
        };

        HttpResponse::Ok().json(
            json_success(articles)
        )
    }

    #[get("/get-articles-from-user")]
    pub async fn get_articles_from_user(conn: web::Data<Connect>, req: HttpRequest) -> impl Responder {
        let user_id = match get_query_param::<i32>(&req, "user_id").await {
            Ok(o) => o,
            Err(e) => return HttpResponse::BadRequest().json(
                json_error(e)
            )
        };

        let articles = match conn.get_articles_from_user(user_id).await {
            Ok(o) => o,
            Err(e) => {
                log(Level::Error, "[GET][get-articles-from-user] >>> conn.get_articles_from_user",
                    &format!("Handle: {}", e)
                );

                return HttpResponse::Ok().json(
                    json_error("Error!")
                );
            },
        };

        HttpResponse::Ok().json(
            json_success(articles)
        )
    }

    #[post("/remove-article")]
    pub async fn remove_article(conn: web::Data<Connect>, mut payload: web::Payload) -> impl Responder {
        let body = match read_body_bytes(&mut payload).await {
            Ok(o) => o,
            Err(_) => return HttpResponse::Ok().json(
                json_error("Request overflow!")
            )
        };

        let value = match serde_json::from_slice::<Value>(&body) {
            Ok(o) => o,
            Err(e) => {
                log(Level::Error, "[POST][remove-article] >>> serde_json::from_slice::<Value>",
                    &format!("Handle: {}", e)
                );

                return HttpResponse::Ok().json(
                    json_error("Error request!")
                )
            },
        };

        let article_id = value["article_id"].as_i64().unwrap() as i32;
        let user_id = value["user_id"].as_i64().unwrap() as i32;

        if let Err(sqlx::Error::RowNotFound) = conn.get_article_info(article_id).await {
            return HttpResponse::Ok().json(
                json_error("Article not found!")
            );
        }

        let is_author = match conn.is_user_author_article(user_id, article_id).await {
            Ok(o) => o,
            Err(e) => {
                log(Level::Error, "[POST][remove-article] >>> conn.is_user_author_article(user_id, article_id)",
                    &format!("Handle: {}", e)
                );

                return HttpResponse::Ok().json(
                    json_error("Error")
                );
            },
        };

        if !is_author {
            return HttpResponse::Ok().json(
                json_error("You are not author this article!")
            );
        }

        if let Err(e) = conn.remove_article(article_id).await {
            log(Level::Error, "[POST][remove-article] >>> conn.remove_article(&article)",
                &format!("Handle: {}", e)
            );

            return HttpResponse::Ok().json(
                json_error("Error")
            );
        }

        HttpResponse::Ok().json(
            json_success("Success")
        )
    }

    #[post("/insert-article-comment")]
    pub async fn insert_article_comment(conn: web::Data<Connect>, mut payload: web::Payload) -> impl Responder {
        let body = match read_body_bytes(&mut payload).await {
            Ok(o) => o,
            Err(_) => return HttpResponse::Ok().json(
                json_error("Request overflow!")
            )
        };

        let insert_comment = match serde_json::from_slice::<InsertComment>(&body) {
            Ok(o) => o,
            Err(e) => {
                log(Level::Error, "[POST][insert-article-comment] >>> serde_json::from_slice::<InsertComment>",
                    &format!("Handle: {}", e)
                );

                return HttpResponse::Ok().json(
                    json_error("Error request!")
                )
            },
        };

        if let Err(sqlx::Error::RowNotFound) = conn.insert_comment_to_article(&insert_comment).await {
            return HttpResponse::Ok().json(
                json_error("Article not found!")
            );
        }

        HttpResponse::Ok().json(
            json_success("Success")
        )
    }

    #[get("/get-article-comments")]
    pub async fn get_article_comments(conn: web::Data<Connect>, req: HttpRequest) -> impl Responder {
        let article_id = match get_query_param::<i32>(&req, "article_id").await {
            Ok(o) => o,
            Err(e) => return HttpResponse::BadRequest().json(
                json_error(e)
            )
        };

        let articles = match conn.get_comments_from_article(article_id).await {
            Ok(o) => o,
            Err(e) => {
                log(Level::Error, "[GET][get-article-comments] >>> conn.get_comments_from_article",
                    &format!("Handle: {}", e)
                );

                return HttpResponse::Ok().json(
                    json_error("Error!")
                );
            },
        };

        HttpResponse::Ok().json(
            json_success(articles)
        )
    }

    #[post("/insert-reaction-for-article")]
    pub async fn insert_reaction_for_article(conn: web::Data<Connect>, mut payload: web::Payload) -> impl Responder {
        let body = match read_body_bytes(&mut payload).await {
            Ok(o) => o,
            Err(_) => return HttpResponse::Ok().json(
                json_error("Request overflow!")
            )
        };

        let reaction = match serde_json::from_slice::<InsertReaction>(&body) {
            Ok(o) => o,
            Err(e) => {
                log(Level::Error, "[POST][insert-reaction-for-article] >>> serde_json::from_slice::<InsertReaction>",
                    &format!("Handle: {}", e)
                );

                return HttpResponse::Ok().json(
                    json_error("Error request!")
                )
            },
        };

        let exists = match conn.exists_reaction_for_article(&reaction).await {
            Ok(o) => o,
            Err(e) => {
                log(Level::Error, "[POST][insert-reaction-for-article] >>> conn.exists_reaction_for_article(&reaction)",
                    &format!("Handle: {}", e)
                );

                return HttpResponse::Ok().json(
                    json_error("Error")
                );
            }
        };

        if exists {
            return HttpResponse::Ok().json(
                json_error("Reaction already inserted!")
            );
        }

        if let Err(e) = conn.insert_reaction_for_article(&reaction).await {
            log(Level::Error, "[POST][insert-reaction-for-article] >>> conn.insert_reaction_for_article(&reaction)",
                &format!("Handle: {}", e)
            );

            return HttpResponse::Ok().json(
                json_error("Error")
            );
        }

        HttpResponse::Ok().json(
            json_success("Success")
        )
    }

    #[get("/get-reaction-for-article-by-user")]
    pub async fn get_reaction_for_article_by_user(conn: web::Data<Connect>, req: HttpRequest) -> impl Responder {
        let user_id = match get_query_param::<i32>(&req, "user_id").await {
            Ok(o) => o,
            Err(e) => return HttpResponse::BadRequest().json(
                json_error(e)
            )
        };

        let article_id = match get_query_param::<i32>(&req, "article_id").await {
            Ok(o) => o,
            Err(e) => return HttpResponse::BadRequest().json(
                json_error(e)
            )
        };

        let name = match conn.get_reaction_for_article_by_user(user_id, article_id).await {
            Ok(o) => o,
            Err(e) => {
                log(Level::Error, "[GET][get-reaction-for-article-by-user] >>> conn.get_reaction_for_article_by_user",
                    &format!("Handle: {}", e)
                );

                return HttpResponse::Ok().json(
                    json_error("Error!")
                );
            },
        };

        HttpResponse::Ok().json(
            json_success(name)
        )
    }

    #[post("/remove-reaction-for-article")]
    pub async fn remove_reaction_for_article(conn: web::Data<Connect>, mut payload: web::Payload) -> impl Responder {
        let body = match read_body_bytes(&mut payload).await {
            Ok(o) => o,
            Err(_) => return HttpResponse::Ok().json(
                json_error("Request overflow!")
            )
        };

        let reaction = match serde_json::from_slice::<InsertReaction>(&body) {
            Ok(o) => o,
            Err(e) => {
                log(Level::Error, "[POST][remove-reaction-for-article] >>> serde_json::from_slice::<InsertReaction>",
                    &format!("Handle: {}", e)
                );

                return HttpResponse::Ok().json(
                    json_error("Error request!")
                )
            },
        };

        let exists = match conn.exists_reaction_for_article(&reaction).await {
            Ok(o) => o,
            Err(e) => {
                log(Level::Error, "[POST][remove-reaction-for-article] >>> conn.exists_reaction_for_article(&reaction)",
                    &format!("Handle: {}", e)
                );

                return HttpResponse::Ok().json(
                    json_error("Error")
                );
            }
        };

        if !exists {
            return HttpResponse::Ok().json(
                json_error("Reaction not found!")
            );
        }

        if let Err(e) = conn.remove_reaction_for_article(&reaction).await {
            log(Level::Error, "[POST][remove-reaction-for-article] >>> conn.remove_reaction_for_article(&reaction)",
                &format!("Handle: {}", e)
            );

            return HttpResponse::Ok().json(
                json_error("Error")
            );
        }

        HttpResponse::Ok().json(
            json_success("Success")
        )
    }
}
