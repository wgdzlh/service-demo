use utoipa::OpenApi;

use crate::entity::{Post, Todo};
use crate::interface::handler::*;
use crate::interface::resp::*;
use crate::repository::*;

#[derive(OpenApi)]
#[openapi(
        servers((url = "/service-demo/v1")),
        paths(
            todo::list,
            todo::create,
            todo::mark_done,
            todo::edit,
            todo::delete,

            post::list,
            post::get,
            post::create,
            post::edit,
            post::delete,

            read_xls::parse,
        ),
        components(
            schemas(IdData, Void, VoidRes,
                Todo, TodoUpdate,
                Post, PostNew, PostList, PostUpdate,
            )
        ),
        // modifiers(&SecurityAddon),
        // tags(
        //     (name = "todo", description = "Todo items management API")
        // )
    )]
pub struct ApiDoc;
