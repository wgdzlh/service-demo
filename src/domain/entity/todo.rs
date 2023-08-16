use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Item to do
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct Todo {
    #[serde(skip_deserializing)]
    #[schema(read_only, example = 1)]
    pub id: i32,
    #[schema(example = "Buy groceries")]
    pub value: String,
    #[schema(example = false)]
    pub done: bool,
}
