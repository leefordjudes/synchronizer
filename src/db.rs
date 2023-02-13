use mongodb::{options::ClientOptions, Client, Database};
use once_cell::sync::OnceCell;

static DB_CLIENT: OnceCell<Client> = OnceCell::new();

pub async fn initialize(uri: &str) {
    let mut client_options = match ClientOptions::parse(&uri).await {
        Ok(options) => options,
        Err(_) => panic!("Database connection failure"),
    };
    client_options.app_name = Some("dbwatcher".to_string());

    let client = match Client::with_options(client_options) {
        Ok(client) => client,
        Err(err) => std::panic::panic_any(err),
    };
    DB_CLIENT.set(client).unwrap();
}

pub fn tenant_db(name: &str) -> Database {
    let client = DB_CLIENT.get().unwrap();
    client.database(name)
}
