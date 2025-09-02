use poem::{
    endpoint::{StaticFileEndpoint, StaticFilesEndpoint},
    get, post, handler,
    web::{Data, Json, Path},
     Route,
};


/* use poem::{
    endpoint::{StaticFileEndpoint, StaticFilesEndpoint}, // static file serving
    get, post, handler,               // routing macros
    web::{Data, Json, Path},          // request extractors and JSON wrapper
    EndpointExt, Route,               // core routing types
}; */
use uuid::Uuid;                           // generate UUIDs for resume links
use mongodb::bson::{doc, Document};       // Mongo doc helpers

use crate::model::{
    fetch_draft, json_to_bson_doc, now_rfc3339, AppState, DraftOut, HelloResponse,
    NewDraftResponse, PatchDraft, Error,
};

#[handler]
pub async fn hello(
    Path(name): Path<String>, // path param
) -> Result<Json<HelloResponse>, Error> {
    Ok(Json(HelloResponse { hello: format!("Hello {}", name) }))
}

#[handler] // POST /api/drafts
pub async fn create_draft(Data(state): Data<&AppState>) -> Result<Json<NewDraftResponse>, Error> {
    let uuid = Uuid::new_v4().to_string();     // generate a new uuid
    let now = now_rfc3339();                   // current timestamp
    let doc = doc! {                                          // assemble draft document
        "uuid": &uuid,
        "data": Document::new(),                              // empty object
        "step": 0_i64,
        "status": "draft",
        "created_at": &now,
        "updated_at": &now,
    };
    state.drafts.insert_one(doc, None).await?;                // insert into Mongo
    
    let resume_url = format!(                  // build resume URL for frontend
        "{}/form/{}",
        state.public_base.trim_end_matches('/'),
        uuid
    );
    Ok(Json(NewDraftResponse { id: uuid, resume_url })) // return JSON
}

#[handler] // GET /api/drafts/:uuid
pub async fn get_draft(
    Data(state): Data<&AppState>,
    Path(uuid): Path<String>,
) -> Result<Json<DraftOut>, Error> {
    Ok(Json(fetch_draft(state, &uuid).await?))
}

#[handler] // PATCH /api/drafts/:uuid
pub async fn patch_draft(
    Data(state): Data<&AppState>,              // shared state
    Path(uuid): Path<String>,                  // path param
    Json(body): Json<PatchDraft>,              // JSON body
) -> Result<Json<DraftOut>, Error> {
    let mut set_doc = doc! {                                  // fields to update
        "data": json_to_bson_doc(&body.data),
        "updated_at": now_rfc3339(),
    };
    if let Some(step) = body.step { set_doc.insert("step", step); } // optional step

    let filter = doc! { "uuid": &uuid, "status": "draft" };   // only update drafts
    state.drafts.update_one(filter, doc! { "$set": set_doc }, None).await?; // update

    Ok(Json(fetch_draft(state, &uuid).await?))                 // return latest state
}

#[handler] // POST /api/drafts/:uuid/submit
pub async fn submit_draft(Data(state): Data<&AppState>, Path(uuid): Path<String>)
    -> Result<Json<DraftOut>, Error>
{
    let filter = doc! { "uuid": &uuid };                      // locate doc
    state.drafts.update_one(                                  // mark as submitted
        filter,
        doc! { "$set": { "status": "submitted", "updated_at": now_rfc3339() } },
        None
    ).await?;
    Ok(Json(fetch_draft(state, &uuid).await?))                 // return finalstate
}

#[handler] // GET /api/health (simple health check)
pub async fn health() -> &'static str {
    "ok"                                       // respond with plain text
}

// route builder called from main
pub fn routes() -> Route {
    
    Route::new()
        .at("/api/hello/:name", get(hello))
        .at("/api/health", get(health))        // health endpoint
        .at("/api/drafts", post(create_draft)) // create draft
        .at("/api/drafts/:uuid", get(get_draft).patch(patch_draft)) // load/save
        .at("/api/drafts/:uuid/submit", post(submit_draft)) // submit 
        .at("/favicon.ico", StaticFileEndpoint::new("www/favicon.ico"))
        .nest("/static/", StaticFilesEndpoint::new("www"))
        .at("*", StaticFileEndpoint::new("www/index.html"))
        //.data(state)
}