pub(crate) mod user {
    use chrono::{DateTime, Utc};
    use serde::{Serialize, Deserialize};

    #[derive(Clone, Eq, Hash, PartialEq, Debug, Serialize, Deserialize, sqlx::FromRow)]
    pub struct FullUser {
        #[sqlx(rename = "user_id")]
        pub id: i32,
        pub first_name: String,
        pub last_name: String,
        pub about: Option<String>,
        pub password: String,
        pub login: String,
        pub crop_avatar: Option<Vec<u8>>,
        pub full_avatar: Option<Vec<u8>>,
        pub date_registration: DateTime<Utc>,
    }

    #[derive(Clone, Eq, Hash, PartialEq, Debug, Serialize, Deserialize, sqlx::FromRow)]
    pub struct RegisterUser {
        pub first_name: String,
        pub last_name: String,
        pub about: Option<String>,
        pub password: String,
        pub login: String,
    }

    #[derive(Clone, Eq, Hash, PartialEq, Debug, Serialize, Deserialize, sqlx::FromRow)]
    pub struct User {
        #[sqlx(rename = "user_id")]
        pub id: i32,
        pub first_name: String,
        pub last_name: String,
        pub about: Option<String>,
        pub crop_avatar: Option<Vec<u8>>,
        pub full_avatar: Option<Vec<u8>>,
        pub date_registration: DateTime<Utc>,
    }

    #[derive(Clone, Eq, Hash, PartialEq, Debug, Serialize, Deserialize, sqlx::FromRow)]
    pub struct PopularUser {
        #[sqlx(rename = "user_id")]
        pub id: i32,
        pub first_name: String,
        pub last_name: String,
        pub about: Option<String>,
        pub crop_avatar: Option<Vec<u8>>,
        pub date_registration: DateTime<Utc>,
        pub followers: i64,
    }
}