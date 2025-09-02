use serde::{Deserialize, Serialize};        // derive Serialize/Deserialize (serialize + deserialize (Rust to JSON, Rust to MongoDB)
use time::{OffsetDateTime, format_description::well_known::Rfc3339};                // RFC3339 timestamps
use bson::Bson;                             // BSON enum for conversion
use mongodb::{
    bson::{doc, Document},
    options::{ClientOptions, IndexOptions},
    Client, Collection,
};
use poem::{error::ResponseError, http::StatusCode}; // map our Error -> HTTP status
use thiserror::Error;

// --- Error type for app ---
#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum Error {
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

//-------------------------------
// Data models
//-------------------------------
#[derive(Clone, Serialize, Deserialize)]     // draft stored in memory (will soon update the code in Mongo)
pub struct Draft {
    pub uuid: String,                             // the resume link uuid
    pub data: serde_json::Value,                  // all form data (simple JSON object)
    pub step: i64,                                // step number    
    pub status: String,                           // "draft" | "submitted" which step
    pub created_at: String,                       // timeestamp
    pub updated_at: String,                       // timestamp
}

#[derive(Serialize)]                          // when response for POST /api/drafts
pub struct NewDraftResponse {
    pub id: String,                               // the uuid
    pub resume_url: String,                       // PUBLIC_BASE/form/:uuid take the base from .env and create the full url
}

#[derive(Serialize)]                          // response shape for GET/PATCH/submit updates to frontend questionnaire
pub struct DraftOut {
    pub id: String,                               // expose as "id" to frontend
    pub data: serde_json::Value,                  // current data (simple JSON object)
    pub step: i64,                                // current step
    pub status: String,                           // "draft" | "submitted"
    pub created_at: String,                       // created timestamp
    pub updated_at: String,                       // updated timestamp
}

#[derive(Deserialize)]                        // body for PATCH (partial save)
pub struct PatchDraft {
    pub data: serde_json::Value,                  // whole form object (simple)
    pub step: Option<i64>,                        // optional step update
}

#[derive(Serialize)]
pub struct HelloResponse {
    pub hello: String,
}

//-------------------------------
// App state
//-------------------------------

// type Store = Arc<RwLock<HashMap<String, Draft>>>; // in-memory store type alias uuid -> Draft, protected by RwLock, shared by Arc HashMap = The actual in-memory database: maps uuid to Draft, RwLock = allows concurrent reads or exclusive write, Arc = allows sharing across threads

#[derive(Clone)]
pub struct AppState {
    pub drafts: Collection<Document>,                             // Mongo collection
    pub public_base: String,                     // base url for resume links
}

// ---------------------------
// Helpers
// ---------------------------
pub fn now_rfc3339() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap()
}

pub fn bson_doc_to_json(doc: &Document) -> serde_json::Value {    // BSON -> serde_json
    bson::from_bson::<serde_json::Value>(Bson::Document(doc.clone()))
        .unwrap_or(serde_json::json!({}))
}

pub fn json_to_bson_doc(value: &serde_json::Value) -> Document {  // serde_json -> BSON doc
    bson::to_bson(value)
        .ok()
        .and_then(|b| b.as_document().cloned())
        .unwrap_or_else(Document::new)
}

// shared draft lookup used by multiple handlers
pub async fn fetch_draft(state: &AppState, uuid: &str) -> Result<DraftOut, Error> {
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

//-------------------------------
// DB init
//-------------------------------
pub async fn init_state() -> Result<AppState, Error> {
    let uri = std::env::var("MONGODB_URI").expect("MONGODB_URI is required");
    let db_name = std::env::var("DB_NAME").unwrap_or_else(|_| "meela_poc".into());
    let public_base = std::env::var("PUBLIC_BASE").unwrap_or_else(|_| "http://localhost:5173".into());

    let mut opts = ClientOptions::parse(&uri).await?; // one-arg parse
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