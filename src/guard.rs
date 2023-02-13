// use std::collections::HashMap;
// use std::pin::Pin;
// use std::result;
// use std::str::FromStr;
// use std::task::{Context, Poll};
// use std::{cell::RefCell, rc::Rc};

// use actix_http::HttpMessage;
// use actix_web::{
//     dev::{Service, ServiceRequest, ServiceResponse, Transform},
//     error::Error,
//     HttpResponse,
// };
// use futures::future;

use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServerContext {
    pub org: String,
    pub ofid: ObjectId,
    pub iat: String,
}
/*
impl actix_web::FromRequest for ServerContext {
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn futures::future::Future<Output = result::Result<Self, Self::Error>>>>;

    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_http::Payload) -> Self::Future {
        let req_clone = req.clone();
        let fut = async move {
            if let Some(ctx) = req_clone.extensions().get::<ServerContext>() {
                return Ok(ctx.clone());
            }
            Err(actix_web::error::ErrorUnauthorized(
                "No server context found",
            ))
        };
        Box::pin(fut)
    }
}

#[derive(Debug, Copy, Clone, Default, serde::Serialize)]
pub struct ServerAccess;

impl<S> Transform<S, ServiceRequest> for ServerAccess
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type InitError = ();
    type Transform = ServerContextMiddleware<S>;
    type Future = future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        future::ok(ServerContextMiddleware {
            service: Rc::new(RefCell::new(service)),
        })
    }
}

pub struct ServerContextMiddleware<S> {
    service: Rc<RefCell<S>>,
}

impl<S> Service<ServiceRequest> for ServerContextMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error> + 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type Future = Pin<Box<dyn future::Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let srv = self.service.clone();
        Box::pin(async move {
            let mut status = false;
            let xst = req
                .headers()
                .get("x-server-token")
                .and_then(|x| x.to_str().ok().map(|x| x.to_string()));
            if let Some(xst) = xst {
                let sctx = ServerSession::validate(&xst).await.ok();
                if let Some(sctx) = sctx {
                    req.extensions_mut().insert(sctx);
                    status = true;
                }
            }

            if status {
                let fut = srv.call(req);
                fut.await
            } else {
                // let err = errors::ErrorResponse::new("NA", "UnAuthorized");
                // Ok(req.into_response(HttpResponse::Unauthorized().json(err)))
                Ok(req.into_response(HttpResponse::Unauthorized().json("UnAuthorized")))
            }
        })
    }
}
*/
