use megacommerce_proto::service::auth::v3::{
  authorization_server::Authorization, CheckRequest, CheckResponse,
};
use megacommerce_shared::utils::time::time_get_seconds;
use tonic::{Code, Request, Response, Status};

use crate::utils::net::extract_jwt_claims_from_request;

use super::{
  hydra::{HydraClient, HydraValidation},
  redis::{RedisCheck, RedisClient},
  response::CheckResponseExt,
  routes::ROUTES,
  Controller,
};

#[tonic::async_trait]
impl Authorization for Controller {
  #[doc = " Performs authorization check based on the attributes associated with the"]
  #[doc = " incoming request, and returns status `OK` or not `OK`."]
  async fn check(&self, request: Request<CheckRequest>) -> Result<Response<CheckResponse>, Status> {
    let ctx = self.get_context(request.get_ref());
    let req = request.get_ref();
    let lang = ctx.accept_language();

    let path = req
      .attributes
      .as_ref()
      .and_then(|a| a.request.as_ref())
      .and_then(|r| r.http.as_ref())
      .map(|h| h.path.clone())
      .ok_or_else(|| Status::new(Code::NotFound, Self::not_found_msg(lang)))?;

    let protected = match ROUTES.get(&path) {
      Some(res) => *res,
      None => return Err(Status::new(Code::NotFound, Self::not_found_msg(lang))),
    };

    if !protected {
      return Ok(self.response_ok(&ctx, &request, None).await);
    }

    let claims = extract_jwt_claims_from_request(&request);
    let token = claims.jti.clone();

    // the token id must be present, for a protected route
    if token.is_empty() {
      return Ok(Response::new(CheckResponse::denied(&Self::invalid_token_msg(lang))));
    }

    match self.redis.check_token(&token).await {
      Ok(RedisCheck::Revoked(_)) => {
        return Ok(Response::new(CheckResponse::denied(&Self::invalid_token_msg(lang))));
      }
      Ok(RedisCheck::Allowed { status }) => {
        let now = time_get_seconds();
        let needs_hydra = match status {
          Some(st) => now as i64 - st.last_checked > 300,
          None => true,
        };

        if needs_hydra {
          // TODO: handle mark_checked_ok, revoke_token errors
          match self.hydra.validate_token(&token).await {
            Ok(HydraValidation::Valid { sub: _, exp: _ }) => {
              self.redis.mark_checked_ok(&token).await.ok();
              return Ok(self.response_ok(&ctx, &request, Some(claims)).await);
            }
            Ok(HydraValidation::Invalid(_)) => {
              self.redis.revoke_token(&token).await.ok();
              return Ok(Response::new(CheckResponse::denied(&Self::invalid_token_msg(lang))));
            }
            Err(err) => {
              self.report_internal_error(err);
              return Err(Status::internal(Self::int_err_msg(lang)));
            }
          }
        } else {
          return Ok(self.response_ok(&ctx, &request, Some(claims)).await); // Cached as valid
        }
      }
      Err(err) => {
        self.report_internal_error(err);
        return Err(Status::internal(Self::int_err_msg(lang)));
      }
    }
  }
}
