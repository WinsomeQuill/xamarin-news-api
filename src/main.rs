use actix_web::{HttpServer, App, web, HttpResponse};
use dotenv::dotenv;
use crate::postgresql::postgresql_manager::Connect;
use crate::services::service_user::user::{
    get_profile_avatar,
    insert_user,
    login_user,
    set_profile_avatar,
    user_info,
    is_user_followed,
    user_count_followers,
    following_user,
    remove_following_user,
};

use crate::services::service_article::article::{insert_article, remove_article, get_articles, get_articles_from_user, get_article_comments, insert_article_comment};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let user = std::env::var("POSTGRES_DB_USER").expect("POSTGRES_DB_USER is invalid!");
    let password = std::env::var("POSTGRES_DB_PASSWORD").expect("POSTGRES_DB_PASSWORD is invalid!");
    let host = std::env::var("POSTGRES_DB_HOST").expect("POSTGRES_DB_HOST is invalid!");
    let port = std::env::var("POSTGRES_DB_PORT").expect("POSTGRES_DB_PORT is invalid!")
        .parse::<u16>()
        .expect("POSTGRES_DB_PORT is not integer!");
    let db_name = std::env::var("POSTGRES_DB_NAME").expect("POSTGRES_DB_NAME is invalid!");

    let postgres = Connect::new(&user, &password, &host, port, &db_name)
        .await
        .expect("[PostgresSQL] Fatal error!");

    println!("[PostgresSQL] Connected!");

    postgres.create_tables()
        .await
        .expect("[PostgresSQL] Error create tables!");

    let data = web::Data::new(postgres);

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            // User service
            .service(insert_user)
            .service(get_profile_avatar)
            .service(login_user)
            .service(set_profile_avatar)
            .service(user_info)
            .service(is_user_followed)
            .service(user_count_followers)
            .service(following_user)
            .service(remove_following_user)
            // Article service
            .service(insert_article)
            .service(remove_article)
            .service(get_articles)
            .service(get_articles_from_user)
            .service(insert_article_comment)
            .service(get_article_comments)
            .default_service(web::to(|| {
                HttpResponse::NotFound()
            }))
        })
        .bind(("192.168.2.104", 27000))?
        .run()
        .await
}

mod postgresql;
mod services;
mod logger;