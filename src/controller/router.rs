use chrono::Local;
use megacommerce_proto::service::auth::v3::{
  authorization_server::Authorization, CheckRequest, CheckResponse,
};
use phf::{phf_map, Map};
use tonic::{Code, Request, Response, Status};

use crate::utils::net::extract_jwt_from_request;

use super::{
  extension::CheckResponseExt,
  hydra::{HydraClient, HydraValidation},
  redis::{RedisCheck, RedisClient},
  Controller,
};

static ROUTES: Map<&'static str, bool> = phf_map! {
  "/user.v1.UsersService/CreateSupplier" =>  false,
};

#[tonic::async_trait]
impl Authorization for Controller {
  #[doc = " Performs authorization check based on the attributes associated with the"]
  #[doc = " incoming request, and returns status `OK` or not `OK`."]
  async fn check(&self, request: Request<CheckRequest>) -> Result<Response<CheckResponse>, Status> {
    let req = request.into_inner();

    let path = req
      .attributes
      .as_ref()
      .and_then(|a| a.request.as_ref())
      .and_then(|r| r.http.as_ref())
      .map(|h| h.path.clone())
      .ok_or_else(|| Status::new(Code::InvalidArgument, "The requested path is not provided!"))?;

    let protected = match ROUTES.get(&path) {
      Some(res) => *res,
      None => return Err(Status::new(Code::NotFound, "The requested resource is not found")),
    };

    if !protected {
      return Ok(Response::new(CheckResponse::ok()));
    }

    let token = extract_jwt_from_request(&req)
      .ok_or_else(|| Status::new(Code::Unauthenticated, "Missing authentication credentials"))?;

    match self.redis.check_token(&token).await {
      Ok(RedisCheck::Revoked(reason)) => {
        return Ok(Response::new(CheckResponse::denied(&format!("revoked: {}", reason))));
      }
      Ok(RedisCheck::Allowed { last_checked }) => {
        let now = Local::now().timestamp();
        let needs_hydra = match last_checked {
          Some(ts) => now - ts > 300,
          None => true,
        };

        if needs_hydra {
          match self.hydra.validate_token(&token).await {
            Ok(HydraValidation::Valid) => {
              self.redis.mark_checked_ok(&token, now).await.ok(); // update timestamp
              return Ok(Response::new(CheckResponse::ok()));
            }
            Ok(HydraValidation::Invalid(reason)) => {
              self.redis.revoke_token(&token).await.ok(); // mark revoked
              return Ok(Response::new(CheckResponse::denied(&reason)));
            }
            Err(err) => return Err(Status::internal(format!("hydra error: {}", err))),
          }
        } else {
          // Cached as valid
          return Ok(Response::new(CheckResponse::ok()));
        }
      }
      Err(err) => return Err(Status::internal(format!("redis error: {}", err))),
    }
  }
}
