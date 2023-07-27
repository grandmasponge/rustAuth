use axum::http::{header, HeaderMap};
use axum::{
    extract::{Json, State},
    http::StatusCode,
    routing::{get, post},
    Router,
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use chrono::{prelude::*, Duration};
use dotenv::dotenv;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
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
    uid: i32,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let secret = std::env::var("SECRET").expect("failed to load env variable");
    //will arc at later stage
    let state = MyState {
        //database connection in a state due to not wating to constantly connect to the database would be rather inefficent if constantly done
        db: Database::connect("mysql://root@localhost:3306/auth")
            .await
            .expect("failed to connect to db"),
        token: secret,
        //look in .env for secret remeber to change the secret when implimenting this
    };
    //adding routes to the app pretty simple stuffs
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
//registering the user
async fn register(
    State(state): State<MyState>,
    Json(payload): Json<User>,
) -> (StatusCode, Json<JsonValue>) {
    let person = m_user::find()
        .filter(users::Column::Username.contains(&payload.username))
        .one(&state.db)
        .await
        .expect("couldnt find user");
    //precense validation for registering the user cant have 2 users with the same details
    match person {
        Some(_) => (
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
            };
            let _user = user.insert(&state.db).await.expect("failed to create user");

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
//the only function in this project that is somewhat been error handled
async fn login(
    State(state): State<MyState>,
    Json(payload): Json<User>,
) -> Result<HeaderMap, (StatusCode, Json<JsonValue>)> {
    let user = m_user::find()
        .filter(users::Column::Username.eq(&payload.username))
        .one(&state.db)
        .await
        .expect("failed to retrive from db");
    //checking user info for logging in
    match user {
        Some(cl) => {
            println!("{:?}", cl);
            if cl.password == payload.password {
                let iat = Utc::now().timestamp() as usize;
                let expt = Utc::now() + Duration::minutes(1);
                let uid = cl.id;
                //setting up the claims aka what data is in the jwt
                let claims = Claims {
                    exp: expt.timestamp() as usize,
                    iat: iat,
                    uid: uid,
                };
                let token = encode(
                    &Header::default(),
                    &claims,
                    &EncodingKey::from_secret(state.token.as_bytes()),
                )
                .expect("failed to create token");
                //creating the cookie
                let cookie = Cookie::build("token", token.to_owned())
                    .path("/")
                    .http_only(true)
                    .same_site(SameSite::Lax)
                    .finish();
                //inserting cookie into headers
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
//todo! but for now juss checks if there is a cookie.
async fn auth(jar: CookieJar, State(state): State<MyState>) {
    if let Some(cookie) = jar.get("token") {
        let token = decode::<Claims>(
            cookie.value(),
            &DecodingKey::from_secret(state.token.as_bytes()),
            &Validation::default(),
        )
        .map_err(|error| match error.kind() {
            jsonwebtoken::errors::ErrorKind::InvalidToken
            | jsonwebtoken::errors::ErrorKind::InvalidSignature
            | jsonwebtoken::errors::ErrorKind::ExpiredSignature => println!("unauthed"),
            _ => println!("also unauthed")
        });
        println!("{:?}", token);
    } else {
        println!("no token found");
    }
}
