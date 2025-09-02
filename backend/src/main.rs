// use std::env;
#[allow(unused_imports)]
use std::{collections::HashMap, sync::Arc}; // HashMap for in-memory store, Arc to share across threads key value store in rust in javascript like map
use log::info;                               // simple logging macro  env_logger::init();

use poem::{                                  // import Poem web items
    EndpointExt, 
    Route, Server,                           // core server & routing types
    endpoint::{StaticFileEndpoint, StaticFilesEndpoint}, // static file serving
    error::ResponseError,                    // map our Error -> HTTP status
    http::StatusCode,                       // HTTP status codes
    get,post, handler,               // routing macros           
    listener::TcpListener,                   // TCP listener
    web::{ Data, Json, Path},               // request extractors and JSON wrapper
};
use serde::{Deserialize, Serialize};        // derive Serialize/Deserialize (serialize + deserialize (Rust to JSON, Rust to MongoDB)
//use tokio::sync::RwLock;                     // async RwLock for our in-memory store
use uuid::Uuid;                           // generate UUIDs for resume links
use time::{OffsetDateTime, format_description::well_known::Rfc3339};                // RFC3339 timestamps
// #use sqlx::SqlitePool;
use bson::Bson;



// --- Mongo/BSON ---
use mongodb::{
    bson::{doc, Document},
    options::{ClientOptions, IndexOptions},
    Client, Collection,
};

#[allow(dead_code)]
#[derive(Debug, thiserror::Error)]
enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    //#[error(transparent)]
    // #Sqlx(#[from] sqlx::Error),
     #[error(transparent)]
    Var(#[from] std::env::VarError),
    #[error(transparent)]
    Dotenv(#[from] dotenv::Error),
    #[error(transparent)] 
    Mongo(#[from] mongodb::error::Error), // Mongo driver errors
    #[error("not found")]
    NotFound,
    #[error("Query failed")]
    QueryFailed,
}

impl ResponseError for Error {
    fn status(&self) -> StatusCode {
       match self {
            Error::NotFound | Error::QueryFailed => StatusCode::NOT_FOUND, // 404
            Error::Io(_) | Error::Var(_) | Error::Dotenv(_) | Error::Mongo(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

/* async fn init_pool() -> Result<SqlitePool, Error> {
    let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;
    Ok(pool)
}
 */

//-------------------------------
// Data models
//-------------------------------

#[derive(Clone, Serialize, Deserialize)]     // draft stored in memory (will soon update the code in Mongo)
struct Draft {
    uuid: String,                             // the resume link uuid
    data: serde_json::Value,                  // all form data (simple JSON object)
    step: i64,                                // step number    
    status: String,                           // "draft" | "submitted" which step
    created_at: String,                       // timeestamp
    updated_at: String,                       // timestamp
}

#[derive(Serialize)]                          // when response for POST /api/drafts
struct NewDraftResponse {
    id: String,                               // the uuid
    resume_url: String,                       // PUBLIC_BASE/form/:uuid take the base from .env and create the full url
}

#[derive(Serialize)]                          // response shape for GET/PATCH/submit updates to frontend questionnaire
struct DraftOut {
    id: String,                               // expose as "id" to frontend
    data: serde_json::Value,                  // current data (simple JSON object)
    step: i64,                                // current step
    status: String,                           // "draft" | "submitted"
    created_at: String,                       // created timestamp
    updated_at: String,                       // updated timestamp
}

#[derive(Deserialize)]                        // body for PATCH (partial save)
struct PatchDraft {
    data: serde_json::Value,                  // whole form object (simple)
    step: Option<i64>,                        // optional step update
}


#[derive(Serialize)]
struct HelloResponse {
    hello: String,
}


//-------------------------------
// App state
//-------------------------------

//type Store = Arc<RwLock<HashMap<String, Draft>>>; // in-memory store type alias uuid -> Draft, protected by RwLock, shared by Arc HashMap = The actual in-memory database: maps uuid to Draft, RwLock = allows concurrent reads or exclusive write, Arc = allows sharing across threads

#[derive(Clone)]
struct AppState {
    drafts: Collection<Document>,                             // Mongo collection
    public_base: String,                     // base url for resume links
}

// ---------------------------
// Helpers
// ---------------------------
fn now_rfc3339() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap()
}

fn bson_doc_to_json(doc: &Document) -> serde_json::Value {    // BSON -> serde_json
    bson::from_bson::<serde_json::Value>(Bson::Document(doc.clone()))
        .unwrap_or(serde_json::json!({}))
}

fn json_to_bson_doc(value: &serde_json::Value) -> Document {  // serde_json -> BSON doc
    bson::to_bson(value)
        .ok()
        .and_then(|b| b.as_document().cloned())
        .unwrap_or_else(Document::new)
}

async fn fetch_draft(state: &AppState, uuid: &str) -> Result<DraftOut, Error> {
    let filter = doc! { "uuid": uuid };
    let found = state
        .drafts
        .find_one(filter, None)
        .await?
        .ok_or(Error::NotFound)?;

    let data = found
        .get_document("data")
        .ok()
        .map(bson_doc_to_json)
        .unwrap_or(serde_json::json!({}));

    let step = found.get_i64("step").unwrap_or(0);
    let status = found.get_str("status").unwrap_or("draft").to_string();
    let created_at = found.get_str("created_at").unwrap_or_default().to_string();
    let updated_at = found.get_str("updated_at").unwrap_or_default().to_string();

    Ok(DraftOut {
        id: uuid.to_string(),
        data,
        step,
        status,
        created_at,
        updated_at,
    })
}

//////////////////////////////////
/////extra///////////////////
//////////////////////////////////
/* fn to_out(d: &Draft) -> DraftOut {             // map internal Draft -> API response
    DraftOut {
        id: d.uuid.clone(),
        data: d.data.clone(),
        step: d.step,
        status: d.status.clone(),
        created_at: d.created_at.clone(),
        updated_at: d.updated_at.clone(),
    }
} */

#[handler]
async fn hello(
    // Data(pool): Data<&SqlitePool>,
    Path(name): Path<String>,
) -> Result<Json<HelloResponse>, Error> {
    /* let r = sqlx::query!("select concat('Hello ', $1) as hello", name)
        .fetch_one(pool)
        .await?;
    let Some(hello) = r.hello else {
        Err(Error::QueryFailed)?
    }; */

    Ok(Json(HelloResponse { hello: format!("Hello {}", name) }))
}


// ---------------------------
// Handlers (these match what we'll keep with Mongo later)
// ---------------------------

#[handler]                                     // POST /api/drafts
async fn create_draft(Data(state): Data<&AppState>) -> Result<Json<NewDraftResponse>, Error> {
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

#[handler]                                     // GET /api/drafts/:uuid
/* async fn get_draft(Data(state): Data<&AppState>, Path(uuid): Path<String>)
    -> Result<Json<DraftOut>, Error>
{
    let filter = doc! { "uuid": &uuid };                      // filter by uuid
    let found = state.drafts.find_one(filter, None).await?    // query one
        .ok_or(Error::NotFound)?;                             // 404 if none

    let data = found.get_document("data").ok()                // extract "data" subdoc
        .map(bson_doc_to_json)                                // convert to JSON
        .unwrap_or(serde_json::json!({}));
    let step = found.get_i64("step").unwrap_or(0);            // step
    let status = found.get_str("status").unwrap_or("draft").to_string(); // status
    let created_at = found.get_str("created_at").unwrap_or_default().to_string();
    let updated_at = found.get_str("updated_at").unwrap_or_default().to_string();

    Ok(Json(DraftOut { id: uuid, data, step, status, created_at, updated_at }))
} */
async fn get_draft(Data(state): Data<&AppState>, Path(uuid): Path<String>)
    -> Result<Json<DraftOut>, Error>
{
    Ok(Json(fetch_draft(state, &uuid).await?))
}


#[handler]                                     // PATCH /api/drafts/:uuid
/* async fn patch_draft(
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

    Ok(Json(fetch_draft(state, &uuid).await?))                  // return latest state

}
 */
async fn patch_draft(
    Data(state): Data<&AppState>,
    Path(uuid): Path<String>,
    Json(body): Json<PatchDraft>,
) -> Result<Json<DraftOut>, Error> {
    let mut set_doc = doc! {
        "data": json_to_bson_doc(&body.data),
        "updated_at": now_rfc3339(),
    };
    if let Some(step) = body.step {
        set_doc.insert("step", step);
    }

    let filter = doc! { "uuid": &uuid, "status": "draft" };
    state
        .drafts
        .update_one(filter, doc! { "$set": set_doc }, None)
        .await?;

    Ok(Json(fetch_draft(state, &uuid).await?))
}

#[handler]                                     // POST /api/drafts/:uuid/submit
async fn submit_draft(Data(state): Data<&AppState>, Path(uuid): Path<String>)
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

#[handler]                                     // GET /api/health (simple health check)
async fn health() -> &'static str {
    "ok"                                       // respond with plain text
}


////////////////'
/////////////////
////////()

async fn init_state() -> Result<AppState, Error> {
    let uri = std::env::var("MONGODB_URI").expect("MONGODB_URI is required");
    let db_name = std::env::var("DB_NAME").unwrap_or_else(|_| "meela_poc".into());
    let public_base = std::env::var("PUBLIC_BASE").unwrap_or_else(|_| "http://localhost:5173".into());

    let mut opts = ClientOptions::parse(&uri).await?;
    opts.app_name = Some("meela-intake-poc".into());

    let client = Client::with_options(opts)?;
    let db = client.database(&db_name);
    let drafts = db.collection::<Document>("drafts");

    // unique index on uuid
    let idx = mongodb::IndexModel::builder()
        .keys(doc! { "uuid": 1 })
        .options(IndexOptions::builder().unique(true).build())
        .build();
    let _ = drafts.create_index(idx, None).await?;

    Ok(AppState { drafts, public_base })
}


#[tokio::main]
async fn main() -> Result<(), Error> {

    println!("listening on http://0.0.0.0:3005");
    dotenv::dotenv().ok();
    env_logger::init();


let state = init_state().await?;


    info!("Listening on http://0.0.0.0:3005");                 // startup log
    // let pool = init_pool().await?;
    let app = Route::new()
        .at("/api/hello/:name", get(hello))
         .at("/api/health", get(health))        // health endpoint
         .at("/api/drafts", post(create_draft)) // create draft
        .at("/api/drafts/:uuid", get(get_draft).patch(patch_draft)) // load/save
        .at("/api/drafts/:uuid/submit", post(submit_draft)) // submit 
        .at("/favicon.ico", StaticFileEndpoint::new("www/favicon.ico"))
        .nest("/static/", StaticFilesEndpoint::new("www"))
        .at("*", StaticFileEndpoint::new("www/index.html"))
        .data(state);
        // .data(pool);
    Server::new(TcpListener::bind("0.0.0.0:3005"))
        .run(app)
        .await?;

    Ok(())
}
