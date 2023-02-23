pub(crate) mod user {
    use chrono::{DateTime, Utc};
    use serde::{Serialize, Deserialize};

    #[derive(Clone, Eq, Hash, PartialEq, Debug, Serialize, Deserialize, sqlx::FromRow)]
    pub struct FullUser {
        pub id: i32,
        pub first_name: String,
        pub last_name: String,
        pub description: Option<String>,
        pub password: String,
        pub login: String,
        pub crop_avatar: Option<String>,
        pub full_avatar: Option<String>,
        pub date_registration: DateTime<Utc>,
    }

    #[derive(Clone, Eq, Hash, PartialEq, Debug, Serialize, Deserialize, sqlx::FromRow)]
    pub struct RegisterUser {
        pub first_name: String,
        pub last_name: String,
        pub description: Option<String>,
        pub password: String,
        pub login: String,
    }

    #[derive(Clone, Eq, Hash, PartialEq, Debug, Serialize, Deserialize, sqlx::FromRow)]
    pub struct User {
        pub id: i32,
        pub first_name: String,
        pub last_name: String,
        pub description: Option<String>,
        pub crop_avatar: Option<String>,
        pub full_avatar: Option<String>,
        pub date_registration: DateTime<Utc>,
    }
}