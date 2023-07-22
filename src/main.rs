use std::sync::Arc;

use axum::{
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
    let state = MyState {db: Database::connect("mysql://root@localhost:3306/auth").await.expect("failed to connect to db")};

    let app = Router::new()
    .route("/register", post(register))
    .with_state(state);

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn register(State(state): State<MyState>, Json(payload): Json<User>) -> Json<JsonValue> {

    let  person = m_user::find()
        .filter(users::Column::Username.contains(&payload.username))
        .one(&state.db)
        .await.expect("couldnt find user");
    
    match(person)
    {
        Some(d) => {
                return Json(json!({"response": 409 ,"data": "exists"}));
            },
        None => {
            return Json(json!({"data": "dont exists"}));
            }
    }

   
}


