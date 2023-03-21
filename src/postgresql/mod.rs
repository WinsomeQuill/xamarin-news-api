pub mod postgresql_manager {
    use base64::Engine;
    use base64::engine::general_purpose;
    use sqlx::{Executor, Pool, Postgres, postgres::PgPoolOptions, Row};
    use super::models;
    use models::model_user::user::{
        User,
        RegisterUser,
    };
    use crate::postgresql::models::model_article::article::{Article, CropArticle, Comment, InsertArticle};


    #[derive(Clone)]
    pub struct Connect {
        pub pool: Pool<Postgres>
    }

    impl Connect {
        /// Создаем новую структуру
        ///
        /// ### Принимает:
        /// Имя пользователя, пароль, адрес хоста, порт хоста, имя базы данных
        ///
        /// Обратите внимание, что порт указывается как u16
        ///
        /// ### Возрващает:
        /// Структуру [`Connet`] или [`sqlx::Error`]
        pub async fn new(user: &str, password: &str, host: &str, port: u16, db_name: &str) -> Result<Connect, sqlx::Error> {
            let mut pool: Result<Pool<Postgres>, sqlx::Error>;
            loop {
                pool = PgPoolOptions::new()
                    .max_connections(5)
                    .acquire_timeout(std::time::Duration::from_millis(10000))
                    .connect(
                        &format!("postgres://{}:{}@{}:{}/{}",
                                 &user, password, host, port, db_name)
                    ).await;

                if pool.is_ok() {
                    break;
                }

                println!("[POSTGRES SQL DB] Timeout connect! Try again...");
                tokio::time::sleep(std::time::Duration::from_millis(5000)).await;
            }

            let pool = pool.unwrap();

            Ok(Connect { pool })
        }

        ///Создаем все нужные таблицы, если их нет.
        pub async fn create_tables(&self) -> Result<(), sqlx::Error> {
            self.pool.execute(r#"
                create table if not exists users (
                    id serial4 primary key,
                    first_name varchar(64) not null,
                    last_name varchar(64) not null,
                    about varchar(256) null,
                    password varchar(64) not null,
                    login varchar(64) not null,
                    full_avatar text null,
                    crop_avatar text null,
                    date_registration timestamptz not null default now()::timestamp with time zone::timestamp
                );

                create table if not exists articles (
                    id serial4 PRIMARY KEY,
                    author_id int4 not null references users(id) on delete cascade,
                    image text not null,
                    title varchar(64) not null,
                    description varchar(1024) not null,
                    publish_date timestamptz not null default now()::timestamp with time zone::timestamp,
                    likes int4 null default 0,
                    dislikes int4 null default 0
                );

                create table if not exists articles_comments (
                    users_id int4 not null references users(id) on delete cascade,
                    articles_id int4 not null references articles(id) on delete cascade,
                    publish_date timestamptz not null default now()::timestamp with time zone::timestamp,
                    message varchar(1024) not null
                );

                create table if not exists users_followers (
                    id serial4 PRIMARY KEY,
                    users_author_id int4 not null references users(id) on delete cascade,
                    users_follower_id int4 not null references users(id) on delete cascade,
                    follow_date timestamptz not null default now()::timestamp with time zone::timestamp
                );
            "#).await?;

            Ok(())
        }

        /// Создаем пользователя в базе данных
        ///
        /// ### Принимает:
        /// Структуру `RegisterUser`
        ///
        /// ### Возвращает:
        /// Если [`Ok`], то `()`. При ошибки [`sqlx::Error`]
        pub async fn insert_user(&self, user: &RegisterUser) -> Result<(), sqlx::Error> {
            let _ = sqlx::query("
                INSERT INTO users (first_name, last_name, about, password, login)
                VALUES ($1, $2, $3, $4, $5)
            ")
                .bind(&user.first_name)
                .bind(&user.last_name)
                .bind(&user.about)
                .bind(&user.password)
                .bind(&user.login)
                .execute(&self.pool).await?;
            Ok(())
        }

        /// Проверяет, есть ли пользователь в базе данных.
        /// ### Принимает:
        /// Логин пользователя
        ///
        /// ### Возвращает:
        /// Если [`Ok`], то `true` - пользователь найден, иначе `false`. В случае ошибки вернется [`slqx::Error`]
        pub async fn exist_user_by_login(&self, login: &str) -> Result<bool, sqlx::Error> {
            let row = sqlx::query("
                SELECT id
                FROM users
                WHERE login = $1
            ")
                .bind(login)
                .fetch_one(&self.pool).await?;

            Ok(row.try_get::<i32, _>("id").is_ok())
        }

        /// Получение данных пользователя по логину и паролю.
        /// ### Принимает:
        /// Логин и пароль пользователя
        ///
        /// ### Возвращает:
        /// Если [`Ok`], то вернется структура `User`. При ошибки [`sqlx::Error`]
        pub async fn get_user_by_login_and_pass(&self, login: &str, password: &str) -> Result<User, sqlx::Error> {
            let row = sqlx::query_as::<_, User>("
                SELECT id AS user_id, first_name, last_name, about, password, login, full_avatar, crop_avatar, date_registration
                FROM users
                WHERE login = $1 AND password = $2
            ")
                .bind(login)
                .bind(password)
                .fetch_one(&self.pool).await?;

            Ok(row)
        }

        /// Получение аватарок пользователя
        /// ### Принимает:
        /// Логин пользователя
        ///
        /// ### Возвращает:
        /// Если [`Ok`], то кортеж из двух String. При ошибки [`sqlx::Error`]
        pub async fn get_avatar_by_login(&self, login: &str) -> Result<(String, String), sqlx::Error> {
            let row = sqlx::query("
                SELECT crop_avatar, full_avatar
                FROM users
                WHERE login = $1
            ")
                .bind(login)
                .fetch_one(&self.pool).await?;

            let crop_avatar: String = match row.try_get("crop_avatar") {
                Ok(o) => o,
                Err(_) => return Ok((String::new(), String::new())),
            };

            let full_avatar: String = match row.try_get("full_avatar") {
                Ok(o) => o,
                Err(_) => return Ok((String::new(), String::new())),
            };

            Ok((crop_avatar, full_avatar))
        }

        /// Установка новых аватаров для пользователя
        /// ### Принимает:
        /// Логин пользователя, маленький аватар и большой аватар
        ///
        /// ### Возвращает:
        /// Если [`Ok`], то `()`. При ошибки [`sqlx::Error`]
        pub async fn set_avatar_by_login(&self, login: &str, crop_avatar: &Vec<u8>, full_avatar: &Vec<u8>) -> Result<(), sqlx::Error> {
            let _ = sqlx::query("
                UPDATE users
                SET crop_avatar = $2, full_avatar = $3
                WHERE login = $1;
            ")
                .bind(login)
                .bind(crop_avatar)
                .bind(full_avatar)
                .fetch_one(&self.pool).await?;

            Ok(())
        }

        /// Получение данных пользователя по ID
        /// ### Принимает:
        /// ID пользователя
        ///
        /// ### Возвращает:
        /// Если [`Ok`], то структура `User`. При ошибки [`sqlx::Error`]
        pub async fn get_user_info_by_id(&self, id: i32) -> Result<User, sqlx::Error> {
            let row = sqlx::query_as::<_, User>("
                SELECT id AS user_id, first_name, last_name, about, password, login, full_avatar, crop_avatar, date_registration
                FROM users
                WHERE id = $1
            ")
                .bind(id)
                .fetch_one(&self.pool).await?;

            Ok(row)
        }

        /// Проверяет, подписан ли пользователь на другого пользователя
        /// ### Принимает:
        /// ID пользователя (автор) и ID пользователя (подписчик)
        ///
        /// ### Возвращает:
        /// Если [`Ok`], то `true` - пользователь подписан, иначе `false`. При ошибки [`sqlx::Error`]
        pub async fn is_user_followed_to_user(&self, author_user_id: i32, follower_user_id: i32) -> Result<bool, sqlx::Error> {
            let row = sqlx::query("
                SELECT *
                FROM users_followers
                WHERE users_author_id = $1
                AND users_follower_id = $2
            ")
                .bind(author_user_id)
                .bind(follower_user_id)
                .fetch_one(&self.pool).await;

            if let Err(sqlx::Error::RowNotFound) = row {
                return Ok(false);
            }

            Ok(row.unwrap().try_get::<i32, _>("id").is_ok())
        }

        /// Получение количество подписчиков пользователя
        /// ### Принимает:
        /// ID пользователя
        ///
        /// ### Возвращает:
        /// Если [`Ok`], то число подписчиков (`i64`). При ошибки [`sqlx::Error`]
        pub async fn get_user_count_followers(&self, user_id: i32) -> Result<i64, sqlx::Error> {
            let row = sqlx::query("
                select COUNT(uf.users_follower_id)
                from users as u, users_followers as uf
                where u.id = $1 and uf.users_author_id = u.id
                group by u.id
            ")
                .bind(user_id)
                .fetch_one(&self.pool).await;

            if let Err(sqlx::Error::RowNotFound) = row {
                return Ok(0);
            }

            if let Ok(o) = row.unwrap().try_get("count") {
                return Ok(o);
            }

            Ok(0)
        }

        /// Создать подписку на пользователя, другому пользователю
        /// ### Принимает:
        /// ID пользователя (автор), ID пользователя (подписчик)
        ///
        /// ### Возвращает:
        /// Если [`Ok`], то `()`. При ошибки [`sqlx::Error`]
        pub async fn set_following_user(&self, author_user_id: i32, follower_user_id: i32) -> Result<(), sqlx::Error> {
            let _ = sqlx::query("
                INSERT INTO
                users_followers (users_author_id, users_follower_id)
                VALUES($1, $2);
            ")
                .bind(author_user_id)
                .bind(follower_user_id)
                .fetch_one(&self.pool).await?;

            Ok(())
        }

        /// Удалить подписку на пользователя
        /// ### Принимает:
        /// ID пользователя (автор), ID пользователя (подписчик)
        ///
        /// ### Возвращает:
        /// Если [`Ok`], то `()`. При ошибки [`sqlx::Error`]
        pub async fn remove_following_user(&self, author_user_id: i32, follower_user_id: i32) -> Result<(), sqlx::Error> {
            let _ = sqlx::query("
                DELETE FROM
                users_followers
                WHERE users_author_id = $1 AND users_follower_id = $2;
            ")
                .bind(author_user_id)
                .bind(follower_user_id)
                .execute(&self.pool).await?;

            Ok(())
        }

        /// Создаем запись в базе данных
        ///
        /// ### Принимает:
        /// Структуру `Article`
        ///
        /// ### Возвращает:
        /// Если [`Ok`], то `()`. При ошибки [`sqlx::Error`]
        pub async fn insert_article(&self, article: &InsertArticle) -> Result<(), sqlx::Error> {
            let image = general_purpose::STANDARD.decode(&article.image).unwrap();
            let _ = sqlx::query("
                INSERT INTO articles
                (author_id, image, title, description)
                VALUES($1, $2, $3, $4);
            ")
                .bind(article.author_id)
                .bind(&image)
                .bind(&article.title)
                .bind(&article.description)
                .execute(&self.pool).await?;
            Ok(())
        }

        /// Получить записи из базе данных
        /// ### Возвращает:
        /// Если [`Ok`], то `Vec<Article>`. При ошибки [`sqlx::Error`]
        pub async fn get_articles(&self) -> Result<Vec<Article>, sqlx::Error> {
            let articles = sqlx::query_as::<_, Article>("
                SELECT a.id AS article_id, image, title, description, publish_date, likes, dislikes,
                u.id AS user_id, first_name, last_name, about, password, login, full_avatar, crop_avatar, date_registration
                FROM articles as a, users as u
                WHERE a.author_id = u.id;
            ")
                .fetch_all(&self.pool)
                .await?;
            Ok(articles)
        }

        /// Получить данные об записи
        /// ### Принимает:
        /// ID записи
        ///
        /// ### Возвращает:
        /// Если [`Ok`], то структура `Article`. При ошибки [`sqlx::Error`]
        pub async fn get_article_info(&self, article_id: i32) -> Result<Article, sqlx::Error> {
            let article = sqlx::query_as::<_, Article>("
                SELECT id AS article_id, author_id, image, title,
                description, publish_date, likes, dislikes
                FROM articles AS a
                WHERE id = $1;
            ")
                .bind(article_id)
                .fetch_one(&self.pool)
                .await?;

            Ok(article)
        }

        /// Удалить запись из базы данных
        /// ### Принимает:
        ///
        /// ID записи
        ///
        /// ### Возвращает:
        /// Если [`Ok`], то `()`. При ошибки [`sqlx::Error`]
        pub async fn remove_article(&self, article_id: i32) -> Result<(), sqlx::Error> {
            let _ = sqlx::query("
                DELETE FROM
                articles
                WHERE id = $1;
            ")
                .bind(article_id)
                .execute(&self.pool).await?;

            Ok(())
        }

        /// Проверка, является ли пользовать создателем записи
        /// ### Принимает:
        ///
        /// ID пользователя, ID записи
        ///
        /// ### Возвращает:
        /// Если [`Ok`], `true` - пользователь является автором записи, иначе `false`. При ошибки [`sqlx::Error`]
        pub async fn is_user_author_article(&self, user_id: i32, article_id: i32) -> Result<bool, sqlx::Error> {
            let row = sqlx::query("
                SELECT id FROM
                articles
                WHERE author_id = $1 AND id = $2;
            ")
                .bind(user_id)
                .bind(article_id)
                .fetch_one(&self.pool).await?;

            Ok(row.try_get::<i32, _>("id").is_ok())
        }

        /// Получение записей определенного пользователя
        /// ### Принимает:
        ///
        /// ID пользователя
        ///
        /// ### Возвращает:
        /// Если [`Ok`], `Vec<Article>`. При ошибки [`sqlx::Error`]
        pub async fn get_articles_from_user(&self, user_id: i32) -> Result<Vec<Article>, sqlx::Error> {
            let row = sqlx::query_as::<_, Article>("
                SELECT a.id AS article_id, image, title, description, publish_date, likes, dislikes,
                u.id AS user_id, first_name, last_name, about, password, login, full_avatar, crop_avatar, date_registration
                FROM articles as a, users as u
                WHERE a.author_id = u.id
                AND u.id = $1;
            ")
                .bind(user_id)
                .fetch_all(&self.pool).await?;

            Ok(row)
        }
    }
}

pub mod models;