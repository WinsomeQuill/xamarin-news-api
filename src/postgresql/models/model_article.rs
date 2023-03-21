pub(crate) mod article {
    use base64::Engine;
    use base64::engine::general_purpose;
    use chrono::{DateTime, Utc};
    use serde::{Serialize, Deserialize, Serializer};
    use crate::postgresql::models::model_user::user::User;

    #[derive(Clone, Eq, Hash, PartialEq, Debug, Serialize, Deserialize, sqlx::FromRow)]
    pub struct Article {
        #[sqlx(rename = "article_id")]
        pub id: i32,
        #[sqlx(flatten)]
        pub author: User,
        pub image: Vec<u8>,
        pub title: String,
        pub description: String,
        pub publish_date: DateTime<Utc>,
        pub likes: i32,
        pub dislikes: i32,
    }

    #[derive(Clone, Eq, Hash, PartialEq, Debug, Serialize, Deserialize, sqlx::FromRow)]
    pub struct CropArticle {
        pub author_id: i32,
        pub image: Vec<u8>,
        pub title: String,
        pub description: String,
    }

    #[derive(Clone, Eq, Hash, PartialEq, Debug, Serialize, Deserialize)]
    pub struct InsertArticle {
        pub author_id: i32,
        pub image: String,
        pub title: String,
        pub description: String,
    }

    #[derive(Clone, Eq, Hash, PartialEq, Debug, Serialize, Deserialize, sqlx::FromRow)]
    pub struct Comment {
        pub id: i32,
        pub user_id: i32,
        pub article_id: String,
        pub message: String,
        pub publish_date: DateTime<Utc>,
    }
}