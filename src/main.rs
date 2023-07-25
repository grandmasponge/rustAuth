use axum::extract::{Json, State};
use axum::http::{header, HeaderMap, HeaderValue, Response};
use axum::response::IntoResponse;
use axum::{
    http::StatusCode,
    routing::{get, post},
    Router,
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use chrono::{prelude::*, Duration};
use dotenv::dotenv;
use jsonwebtoken::{encode, EncodingKey, Header};
use myent::users;
use myent::users::Entity as m_user;
use sea_orm::ActiveValue::{NotSet, Set};
use sea_orm::{
    sea_query::tests_cfg::json, ActiveModelTrait, ColumnTrait, Database, DatabaseConnection,
    EntityTrait, JsonValue, QueryFilter,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct User {
    username: String,
    password: String,
}

#[derive(Clone)]
struct MyState {
    db: DatabaseConnection,
    token: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    exp: usize,
    iat: usize,
    uid: i32
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let secret = std::env::var("SECRET").expect("failed to load env variable");
    let state = MyState {
        db: Database::connect("mysql://root@localhost:3306/auth")
            .await
            .expect("failed to connect to db"),
        token: secret,
    };

    let app = Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/auth", get(auth))
        .with_state(state.clone());

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn register(
    State(state): State<MyState>,
    Json(payload): Json<User>,
) -> (StatusCode, Json<JsonValue>) {
    let person = m_user::find()
        .filter(users::Column::Username.contains(&payload.username))
        .one(&state.db)
        .await
        .expect("couldnt find user");

    match person {
        Some(d) => (
            StatusCode::NOT_ACCEPTABLE,
            Json(json!(
                {"response": 409,
                 "data": "exists"}
            )),
        ),
        None => {
            let user = users::ActiveModel {
                id: NotSet,
                username: Set(payload.username),
                password: Set(payload.password),
                jwt_exp: Set(None),
            };
            let user = user.insert(&state.db).await.expect("failed to create user");

            (
                StatusCode::CREATED,
                Json(json!(
                {
                    "response": 200,
                     "data": "created user"
                    }
                )),
            )
        }
    }
}

async fn login(
    State(state): State<MyState>,
    Json(payload): Json<User>,
) -> Result<HeaderMap, (StatusCode, Json<JsonValue>)> {
    let user = m_user::find()
        .filter(users::Column::Username.contains(&payload.username))
        .one(&state.db)
        .await
        .expect("failed to retrive from db");

    match user {
        Some(mut cl) => {
            if &cl.password == &payload.password {
                let iat = Utc::now().timestamp() as usize;
                let expt = Utc::now() + Duration::hours(1);
                let uid = cl.id;
                let claims = Claims {
                    exp: expt.timestamp() as usize,
                    iat: iat,
                    uid: uid
                };
                let token = encode(
                    &Header::default(),
                    &claims,
                    &EncodingKey::from_secret(state.token.as_bytes()),
                )
                .expect("failed to create token");
                let cookie = Cookie::build("token", token.to_owned())
                    .path("/")
                    .http_only(true)
                    .same_site(SameSite::Lax)
                    .finish();
                let mut headers = HeaderMap::new();
                headers.insert(header::SET_COOKIE, cookie.to_string().parse().unwrap());
                Ok(headers)
            } else {
                Err((
                    StatusCode::UNAUTHORIZED,
                    Json(json!(
                        {
                            "data": "tut tut tut"
                        }
                    )),
                ))
            }
        }
        None => Err((
            StatusCode::UNAUTHORIZED,
            Json(json!(
                {
                    "data": "tut tut tut"
                }
            )),
        )),
    }
}

async fn auth(jar: CookieJar) {
    if let Some(token) = jar.get("token")
    {
        println!("{:?}", token.value());
    }
    else {
        println!("no token found");
    }
}
