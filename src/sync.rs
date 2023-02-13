use actix_web::{error, web, HttpRequest, HttpResponse};
use futures::StreamExt;
// use futures::TryStreamExt;
use mongodb::{
    bson::{
        doc, oid::ObjectId, serde_helpers::bson_datetime_as_rfc3339_string, DateTime,
        RawDocumentBuf,
    },
    Cursor,
};
// use mongodb::bson::Document;
use serde::Deserialize;
use tonic::transport::Channel;

use crate::db;
use crate::{
    auth::{auth_session_client::AuthSessionClient, SessionGetRequest},
    guard::ServerContext,
};

const REQUIRE_COLL: [&str; 16] = [
    "inventories",
    "branches",
    "accounts",
    "gst_registrations",
    "inventory_heads",
    "contacts",
    "doctors",
    "patients",
    "voucher_types",
    "sale_incharges",
    "cash_registers",
    "print_templates",
    "members",
    "units",
    "account_pendings",
    "batches",
];

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SynchronizeInput {
    pub branches: Option<String>,
    #[serde(with = "bson_datetime_as_rfc3339_string")]
    pub synced_at: DateTime,
}

async fn synchronize(
    req: HttpRequest,
    auth_session: web::Data<AuthSessionClient<Channel>>,
    path: web::Path<String>,
    args: web::Query<SynchronizeInput>,
) -> actix_web::Result<HttpResponse> {
    let collection = path.into_inner();
    let args = args.into_inner();
    let server_token = req
        .headers()
        .get("x-server-token")
        .and_then(|x| x.to_str().ok().map(|x| x.to_string()))
        .ok_or_else(|| error::ErrorBadRequest("Server token not found"))?;
    let ses_req = SessionGetRequest {
        token: server_token,
        org: None,
    };
    let payload = auth_session
        .as_ref()
        .clone()
        .offline_server_get(ses_req)
        .await
        .map_err(|_| error::ErrorBadRequest("Invalid server token"))?
        .into_inner()
        .payload;
    let sctx = serde_json::from_str::<ServerContext>(&payload)
        .map_err(|_| error::ErrorBadRequest("Could not get server context"))?;
    let db = &db::tenant_db(&sctx.org);
    let mut bids: Vec<ObjectId> = Vec::new();
    if let Some(ref brs) = args.branches {
        bids = serde_json::from_str(&brs).ok().unwrap_or_default();
    }

    let mut filter = doc! {"updatedAt": {"$lte": &args.synced_at}};
    if !REQUIRE_COLL.contains(&collection.as_str()) {
        return Err(error::ErrorBadRequest("Could not get collection instance"));
    }
    // let docs = db
    //     .collection::<Document>(&collection)
    //     .find(filter.clone(), None)
    //     .await
    //     .map_err(|_| error::ErrorInternalServerError("err1"))?
    //     .try_collect::<Vec<Document>>()
    //     .await
    //     .map_err(|_| error::ErrorInternalServerError("err2"))?;
    // println!("docs: {:?}", docs);
    let result = db
        .collection::<RawDocumentBuf>(&collection)
        .find(filter, None)
        .await
        .map_err(|_| error::ErrorInternalServerError("Can't get data"));
    let data: Cursor<RawDocumentBuf>;
    match result {
        Ok(x) => data = x,
        Err(_) => return Err(error::ErrorInternalServerError("Can't get data")),
    }

    let stream = data.map(|x| x.map(|y| web::Bytes::copy_from_slice(y.as_bytes())));
    Ok(HttpResponse::Ok().streaming(stream))
}

pub fn init_routes(route_path: &str, cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource(format!("{}/synchronize/{{collection}}", route_path))
            .route(web::get().to(synchronize)), // .wrap(ServerAccess),
    );
}
