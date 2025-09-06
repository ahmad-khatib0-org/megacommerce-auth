use megacommerce_proto::service::auth::v3::{
  authorization_server::Authorization, CheckRequest, CheckResponse,
};
use megacommerce_shared::utils::time::time_get_seconds;
use phf::{phf_map, Map};
use tonic::{Code, Request, Response, Status};

use crate::utils::net::extract_jwt_claims_from_request;

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
    let req = request.get_ref();

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

    let token = extract_jwt_claims_from_request(&request).jti;

    match self.redis.check_token(&token).await {
      Ok(RedisCheck::Revoked(reason)) => {
        return Ok(Response::new(CheckResponse::denied(&format!("revoked: {}", reason))));
      }
      Ok(RedisCheck::Allowed { status }) => {
        let now = time_get_seconds();
        let needs_hydra = match status {
          Some(st) => now as i64 - st.last_checked > 300,
          None => true,
        };

        if needs_hydra {
          match self.hydra.validate_token(&token).await {
            Ok(HydraValidation::Valid) => {
              self.redis.mark_checked_ok(&token).await.ok(); // update timestamp
              return Ok(Response::new(CheckResponse::ok()));
            }
            Ok(HydraValidation::Invalid(reason)) => {
              self.redis.revoke_token(&token).await.ok(); // mark revoked
              return Ok(Response::new(CheckResponse::denied(&reason)));
            }
            Err(err) => return Err(Status::internal(format!("hydra error: {}", err))),
          }
        } else {
          return Ok(Response::new(CheckResponse::ok())); // Cached as valid
        }
      }
      Err(err) => return Err(Status::internal(format!("redis error: {}", err))),
    }
  }
}
