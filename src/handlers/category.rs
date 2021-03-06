use actix_web::{HttpRequest, HttpResponse, Responder, web};
use serde::{Serialize, Deserialize};

use crate::common::WebAppData;
use crate::errors::{ServiceError, ServiceResult};
use crate::models::response::{CategoryResponse, OkResponse};

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/category")
            .service(web::resource("")
                .route(web::get().to(get_categories))
                .route(web::post().to(add_category))
                .route(web::delete().to(delete_category))
            )
    );
}

pub async fn get_categories(app_data: WebAppData) -> ServiceResult<impl Responder> {
    // Count torrents with category
    let res = sqlx::query_as::<_, CategoryResponse>(
        r#"SELECT name, COUNT(tt.category_id) as num_torrents
           FROM torrust_categories tc
           LEFT JOIN torrust_torrents tt on tc.category_id = tt.category_id
           GROUP BY tc.name"#
    )
        .fetch_all(&app_data.database.pool)
        .await?;

    Ok(HttpResponse::Ok().json(OkResponse {
        data: res
    }))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Category {
    pub name: String
}

pub async fn add_category(req: HttpRequest, payload: web::Json<Category>, app_data: WebAppData) -> ServiceResult<impl Responder> {
    // check for user
    let user = app_data.auth.get_user_from_request(&req).await?;

    // check if user is administrator
    if !user.administrator { return Err(ServiceError::Unauthorized) }

    let res = sqlx::query!(
        "INSERT INTO torrust_categories (name) VALUES ($1)",
        payload.name,
    )
        .execute(&app_data.database.pool)
        .await;

    if let Err(sqlx::Error::Database(err)) = res {
        return if err.message().contains("UNIQUE") {
            Err(ServiceError::CategoryExists)
        } else {
            Err(ServiceError::InternalServerError)
        }
    }

    Ok(HttpResponse::Ok().json(OkResponse {
        data: payload.name.clone()
    }))
}

pub async fn delete_category(req: HttpRequest, payload: web::Json<Category>, app_data: WebAppData) -> ServiceResult<impl Responder> {
    // check for user
    let user = app_data.auth.get_user_from_request(&req).await?;

    // check if user is administrator
    if !user.administrator { return Err(ServiceError::Unauthorized) }

    let _res = sqlx::query!(
        "DELETE FROM torrust_categories WHERE name = $1",
        payload.name,
    )
        .execute(&app_data.database.pool)
        .await?;

    Ok(HttpResponse::Ok().json(OkResponse {
        data: payload.name.clone()
    }))
}
