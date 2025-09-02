mod api;
mod model;

use log::info;                               // simple logging macro  env_logger::init();
use poem::{listener::TcpListener, EndpointExt, Server};

#[tokio::main]
async fn main() -> Result<(), model::Error> {
    println!("listening on http://0.0.0.0:3005");
    dotenv::dotenv().ok();
    env_logger::init();

    let state = model::init_state().await?;
    let app = api::routes().data(state);

    info!("Listening on http://0.0.0.0:3005");                 // startup log
    Server::new(TcpListener::bind("0.0.0.0:3005"))
        .run(app)
        .await?;

    Ok(())
}