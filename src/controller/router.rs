use megacommerce_proto::service::auth::v3::{
  authorization_server::Authorization, CheckRequest, CheckResponse,
};
use phf::{phf_map, Map};
use tonic::{Code, Request, Response, Status};

use super::Controller;

static ROUTES: Map<&'static str, bool> = phf_map! {
  "/user.v1.UsersService/CreateSupplier" =>  false,
};

#[tonic::async_trait]
impl Authorization for Controller {
  #[doc = " Performs authorization check based on the attributes associated with the"]
  #[doc = " incoming request, and returns status `OK` or not `OK`."]
  async fn check(&self, request: Request<CheckRequest>) -> Result<Response<CheckResponse>, Status> {
    let ext = request.extensions();
    let req = request.into_inner();

    let path = req.attributes.unwrap().request.unwrap().http.unwrap().path;

    let protected = match ROUTES.get(&path) {
      Some(res) => *res,
      None => return Err(Status::new(Code::NotFound, "The requested resource is not found")),
    };

    Ok(Response::new(CheckResponse::default()))
  }
}
