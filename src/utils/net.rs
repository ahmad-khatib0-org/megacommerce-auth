use std::io::{Error, ErrorKind};

use http::Uri;

pub fn validate_url_target(url: &str) -> Result<Uri, Error> {
  url
    .parse::<Uri>()
    .map_err(|e| Error::new(ErrorKind::InvalidInput, format!("invalid URL: {}", e)))
}
