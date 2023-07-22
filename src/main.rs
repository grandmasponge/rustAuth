use std::sync::Arc;

use axum::{
    http::StatusCode,
    Router,
     routing::{get, post},
    };
use axum::extract::{Json, State};
use serde::Deserialize;
use sea_orm::{DatabaseConnection, Database, ActiveModelTrait, EntityTrait, QueryFilter, ColumnTrait, sea_query::tests_cfg::json, JsonValue};
use sea_orm::ActiveValue::{Set, NotSet, Unchanged};
use myent::users;
use myent::users::Entity as m_user;
#[derive(Debug,Deserialize)]
struct User {
    username: String,
    password: String,
}
#[derive(Clone)]
struct MyState{
    db: DatabaseConnection
}

#[tokio::main]
async fn main() {
    let state = MyState {
        db: Database::connect("mysql://root@localhost:3306/auth")
        .await
        .expect("failed to connect to db")};

    let app = Router::new()
    .route("/register", post(register))
    .with_state(state.clone())
    .route("/login", post(login))
    .with_state(state.clone());
    
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn register(State(state): State<MyState>, Json(payload): Json<User>) -> (StatusCode, Json<JsonValue>) {

    let  person = m_user::find()
        .filter(users::Column::Username.contains(&payload.username))
        .one(&state.db)
        .await.expect("couldnt find user");
    
    match person
    {
        Some(d) => {
                (StatusCode::NOT_ACCEPTABLE,
                    Json(
                        json!(
                            {"response": 409,
                             "data": "exists"}
                        )
                     )
                )
            },
        None => {
                let user = users::ActiveModel{
                    id: NotSet,
                    username: Set(payload.username),
                    password: Set(payload.password),
                    jwt_exp: Set(None)
                };
                let user = user
                .insert(&state.db)
                .await
                .expect("failed to create user");

                 (StatusCode::CREATED,
                     Json(
                        json!(
                            {
                                "response": 200,
                                 "data": "created user"
                                }
                            )
                        )
                    )
            }
    }

   
}

async fn login(State(state): State<MyState>, Json(payload): Json<User>) -> (StatusCode, Json<JsonValue>) {
    let user = m_user::find()
                .filter(
                    users::Column::Username.contains(&payload.username)
                )
                .one(&state.db)
                .await
                .expect("failed to retrive from db");
    match user {
        Some(cl) => {
                if &cl.password == &payload.password {
                    todo!()
                }
                else {
                    (StatusCode::UNAUTHORIZED, Json(json!({"data": "tut tut tut"})))
                }
        },
        None => {
            (StatusCode::UNAUTHORIZED, Json(json!({"data": "tut tut tut"})))
        }
    }           
}
