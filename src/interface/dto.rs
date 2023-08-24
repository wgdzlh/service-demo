use utoipa::ToSchema;

#[derive(ToSchema)]
pub struct MultipartFile {
    // #[schema(value_type = String, format = Binary)]
    pub file: Vec<u8>,
}
