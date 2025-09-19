#[derive(Debug)]
pub struct EssentialHttpHeaders {
  pub path: String,
  pub method: String,
  pub user_agent: String,
  pub x_forwarded_for: String,
  pub x_request_id: String,
  pub accept_language: String,
  pub headers: std::collections::HashMap<String, String>,
}
