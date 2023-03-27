pub mod postgresql_manager {
    use base64::Engine;
    use base64::engine::general_purpose;
    use sqlx::{Executor, Pool, Postgres, postgres::PgPoolOptions, Row};
    use super::models;
    use models::model_user::user::{
        User,
        RegisterUser,
    };
    use crate::postgresql::models::model_article::article::{
        Article,
        Comment,
        InsertArticle,
        InsertComment,
        InsertReaction
    };
    use crate::postgresql::models::model_user::user::PopularUser;


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
                CREATE TABLE IF NOT EXISTS users (
                    id serial4 PRIMARY KEY,
                    first_name varchar(64) NOT NULL,
                    last_name varchar(64) NOT NULL,
                    about varchar(256) NULL,
                    password varchar(64) NOT NULL,
                    login varchar(64) NOT NULL,
                    full_avatar text NULL,
                    crop_avatar text NULL,
                    date_registration timestamptz NOT NULL default now()::timestamp with time zone::timestamp
                );

                CREATE TABLE IF NOT EXISTS articles (
                    id serial4 PRIMARY KEY,
                    author_id int4 NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                    image text NOT NULL,
                    title varchar(64) NOT NULL,
                    description varchar(1024) NOT NULL,
                    publish_date timestamptz NOT NULL default now()::timestamp with time zone::timestamp
                );

                CREATE TABLE IF NOT EXISTS articles_comments (
                    id serial4 PRIMARY KEY,
                    users_id int4 NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                    articles_id int4 NOT NULL REFERENCES articles(id) ON DELETE CASCADE,
                    publish_date timestamptz NOT NULL default now()::timestamp with time zone::timestamp,
                    message varchar(1024) NOT NULL
                );

                CREATE TABLE IF NOT EXISTS users_followers (
                    id serial4 PRIMARY KEY,
                    users_author_id int4 NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                    users_follower_id int4 NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                    follow_date timestamptz NOT NULL default now()::timestamp with time zone::timestamp
                );

                CREATE TABLE IF NOT EXISTS reactions (
                    id serial4 PRIMARY KEY,
                    description varchar(16) NOT NULL
                );

                INSERT INTO reactions (description)
                SELECT 'Нравится'
                WHERE NOT EXISTS (
                    SELECT description FROM reactions WHERE description = 'Нравится'
                );

                INSERT INTO reactions (description)
                SELECT 'Не нравится'
                WHERE NOT EXISTS (
                    SELECT description FROM reactions WHERE description = 'Не нравится'
                );

                CREATE TABLE IF NOT EXISTS articles_reactions (
                    id serial4 PRIMARY KEY,
                    users_id int4 NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                    articles_id int4 NOT NULL REFERENCES articles(id) ON DELETE CASCADE,
                    reactions_id int4 NOT NULL REFERENCES reactions(id) ON DELETE CASCADE,
                    date timestamptz NOT NULL default now()::timestamp with time zone::timestamp
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
        /// Если [`Ok`], то число подписчиков `i64`. При ошибки [`sqlx::Error`]
        pub async fn get_user_count_followers(&self, user_id: i32) -> Result<i64, sqlx::Error> {
            let row = sqlx::query("
                SELECT COUNT(uf.users_follower_id)
                FROM users AS u, users_followers AS uf
                WHERE uf.users_author_id = $1 AND uf.users_author_id = u.id;
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
            let mut articles = sqlx::query_as::<_, Article>("
                SELECT a.id AS article_id, image, title, description AS full_description,
                CONCAT(LEFT(description, 150), '...') AS crop_description, publish_date,
                u.id AS user_id, first_name, last_name, about, password, login, full_avatar, crop_avatar, date_registration
                FROM articles as a, users as u
                WHERE a.author_id = u.id;
            ")
                .fetch_all(&self.pool)
                .await?;

            for article in &mut articles {
                let (likes, dislikes) = self.get_reactions_from_article(article.id).await.unwrap();
                article.likes = likes;
                article.dislikes = dislikes;
            }

            Ok(articles)
        }

        /// Получить данные об записи
        /// ### Принимает:
        /// ID записи
        ///
        /// ### Возвращает:
        /// Если [`Ok`], то структура `Article`. При ошибки [`sqlx::Error`]
        pub async fn get_article_info(&self, article_id: i32) -> Result<Article, sqlx::Error> {
            let mut article = sqlx::query_as::<_, Article>("
                SELECT id AS article_id, author_id, image, title,
                description AS full_description, concat(left(description, 150), '...') AS crop_description, publish_date
                FROM articles AS a
                WHERE id = $1;
            ")
                .bind(article_id)
                .fetch_one(&self.pool)
                .await?;

            let (likes, dislikes) = self.get_reactions_from_article(article.id).await.unwrap();
            article.likes = likes;
            article.dislikes = dislikes;

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
        /// Если [`Ok`], то `true` - пользователь является автором записи, иначе `false`. При ошибки [`sqlx::Error`]
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
        /// Если [`Ok`], то `Vec<Article>`. При ошибки [`sqlx::Error`]
        pub async fn get_articles_from_user(&self, user_id: i32) -> Result<Vec<Article>, sqlx::Error> {
            let mut articles = sqlx::query_as::<_, Article>("
                SELECT a.id AS article_id, image, title, description AS full_description,
                CONCAT(LEFT(description, 150), '...') AS crop_description, publish_date,
                u.id AS user_id, first_name, last_name, about, password, login, full_avatar, crop_avatar, date_registration
                FROM articles as a, users as u
                WHERE a.author_id = u.id
                AND u.id = $1;
            ")
                .bind(user_id)
                .fetch_all(&self.pool).await?;

            for article in &mut articles {
                let (likes, dislikes) = self.get_reactions_from_article(article.id).await.unwrap();
                article.likes = likes;
                article.dislikes = dislikes;
            }

            Ok(articles)
        }

        /// Создаем комментарий к записи в базе данных
        ///
        /// ### Принимает:
        /// Структуру `InsertComment`
        ///
        /// ### Возвращает:
        /// Если [`Ok`], то `()`. При ошибки [`sqlx::Error`]
        pub async fn insert_comment_to_article(&self, comment: &InsertComment) -> Result<(), sqlx::Error> {
            let _ = sqlx::query("
                INSERT INTO articles_comments
                (users_id, articles_id, message)
                VALUES($1, $2, $3);
            ")
                .bind(comment.user_id)
                .bind(comment.article_id)
                .bind(&comment.message)
                .execute(&self.pool).await?;
            Ok(())
        }

        /// Получение комментариев к записи
        /// ### Принимает:
        ///
        /// ID записи
        ///
        /// ### Возвращает:
        /// Если [`Ok`], то `Vec<Comment>`. При ошибки [`sqlx::Error`]
        pub async fn get_comments_from_article(&self, article_id: i32) -> Result<Vec<Comment>, sqlx::Error> {
            let row = sqlx::query_as::<_, Comment>("
                SELECT ac.id AS id, u.id AS user_id, first_name, last_name, about, crop_avatar, full_avatar, date_registration, message, publish_date
                FROM articles_comments AS ac, users AS u
                WHERE ac.users_id = u.id AND ac.articles_id = $1;
            ")
                .bind(article_id)
                .fetch_all(&self.pool).await?;

            Ok(row)
        }

        /// Получение реакций к записи
        /// ### Принимает:
        ///
        /// ID записи
        ///
        /// ### Возвращает:
        /// Если [`Ok`], то `(i64, i64)`. При ошибки [`sqlx::Error`]
        pub async fn get_reactions_from_article(&self, article_id: i32) -> Result<(i64, i64), sqlx::Error> {
            let row = sqlx::query("
                SELECT COUNT(ar.id)
                FROM articles_reactions AS ar, reactions AS r
                WHERE ar.articles_id = $1 AND ar.reactions_id = r.id AND r.description = 'Нравится';
            ")
                .bind(article_id)
                .fetch_one(&self.pool).await?;

            let likes: i64 = row.try_get("count").unwrap();

            let row = sqlx::query("
                SELECT COUNT(ar.id)
                FROM articles_reactions AS ar, reactions AS r
                WHERE ar.articles_id = $1 AND ar.reactions_id = r.id AND r.description = 'Не нравится';
            ")
                .bind(article_id)
                .fetch_one(&self.pool).await?;

            let dislikes: i64 = row.try_get("count").unwrap();

            Ok((likes, dislikes))
        }

        /// Создание реакции к записи
        /// ### Принимает:
        ///
        /// Структуру `InsertReaction`
        ///
        /// ### Возвращает:
        /// Если [`Ok`], то `()`. При ошибки [`sqlx::Error`]
        pub async fn insert_reaction_for_article(&self, reaction: &InsertReaction) -> Result<(), sqlx::Error> {
            let _ = sqlx::query("
                INSERT INTO articles_reactions
                (users_id, articles_id, reactions_id)
                VALUES($1, $2, (SELECT id FROM reactions WHERE description = $3));
            ")
                .bind(reaction.user_id)
                .bind(reaction.article_id)
                .bind(&reaction.reaction)
                .execute(&self.pool).await?;

            Ok(())
        }

        /// Удаление реакции к записи
        /// ### Принимает:
        ///
        /// Структуру `InsertReaction`
        ///
        /// ### Возвращает:
        /// Если [`Ok`], то `()`. При ошибки [`sqlx::Error`]
        pub async fn remove_reaction_for_article(&self, reaction: &InsertReaction) -> Result<(), sqlx::Error> {
            let _ = sqlx::query("
                DELETE FROM articles_reactions AS ar
                WHERE ar.users_id = $1 AND ar.articles_id = $2
            ")
                .bind(reaction.user_id)
                .bind(reaction.article_id)
                .execute(&self.pool).await?;

            Ok(())
        }

        /// Проверка существования реакции к записи
        /// ### Принимает:
        ///
        /// Структуру `InsertReaction`
        ///
        /// ### Возвращает:
        /// Если [`Ok`], то `bool`. При ошибки [`sqlx::Error`]
        pub async fn exists_reaction_for_article(&self, reaction: &InsertReaction) -> Result<bool, sqlx::Error> {
            let row = sqlx::query("
                SELECT ar.id
                FROM articles_reactions AS ar
                WHERE ar.users_id = $1 AND ar.articles_id = $2
            ")
                .bind(reaction.user_id)
                .bind(reaction.article_id)
                .fetch_one(&self.pool).await;

            if let Err(sqlx::Error::RowNotFound) = row {
                return Ok(false);
            }

            let row = row.unwrap();

            Ok(row.try_get::<i32, _>("id").is_ok())
        }

        /// Получить реакцию пользователя на запись
        /// ### Принимает:
        ///
        /// ID пользователя, ID записи
        ///
        /// ### Возвращает:
        /// Если [`Ok`], то `Option<String>`. При ошибки [`sqlx::Error`]
        pub async fn get_reaction_for_article_by_user(&self, user_id: i32, article_id: i32) -> Result<Option<String>, sqlx::Error> {
            let row = sqlx::query("
                SELECT r.description
                FROM articles_reactions AS ar, reactions AS r
                WHERE ar.users_id = $1 AND ar.articles_id = $2 AND ar.reactions_id = r.id
            ")
                .bind(user_id)
                .bind(article_id)
                .fetch_one(&self.pool).await;

            if let Err(sqlx::Error::RowNotFound) = row {
                return Ok(None);
            }

            let row = row.unwrap();

            if row.try_get::<String, _>("description").is_err() {
                return Ok(None);
            }

            Ok(Some(row.get("description")))
        }

        /// Получить популярных пользователей
        /// ### Принимает:
        ///
        /// ID пользователя
        ///
        /// ### Возвращает:
        /// Если [`Ok`], то `Vec<User>`. При ошибки [`sqlx::Error`]
        pub async fn get_popular_users(&self, user_id: i32) -> Result<Vec<PopularUser>, sqlx::Error> {
            let row = sqlx::query_as::<_, PopularUser>("
                SELECT COUNT(u.id) AS followers, u.id AS user_id, first_name, last_name, about,
                password, login, full_avatar, crop_avatar, date_registration
                FROM users AS u, users_followers AS uf
                WHERE uf.users_author_id = u.id AND u.id != $1
                GROUP BY u.id
                ORDER BY followers DESC
            ")
                .bind(user_id)
                .fetch_all(&self.pool).await;

            if let Err(sqlx::Error::RowNotFound) = row {
                return Ok(Vec::with_capacity(0));
            }

            Ok(row.unwrap())
        }
    }
}

pub mod models;