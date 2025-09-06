use megacommerce_proto::AppError;

fn sanitize_app_error(err: &AppError) -> AppError {
  AppError {
    id: err.id.clone(),
    message: err.message.clone(),
    request_id: err.request_id.clone(),
    status_code: err.status_code,
    skip_translation: err.skip_translation,
    errors: err.errors.clone(),
    errors_nested: err.errors_nested.clone(),
    // scrub sensitive fields
    r#where: String::new(),
    detailed_error: String::new(),
  }
}
