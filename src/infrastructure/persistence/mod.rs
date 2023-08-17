mod todo;
pub use todo::TodoStore;

mod post;
pub use post::PostStore;

use sea_orm::DbConn;

pub struct Db {
    pub todo: TodoStore,
    pub post: PostStore,
}

impl Db {
    pub fn new(db: &DbConn) -> Self {
        Self {
            todo: todo::get_todo_store(),
            post: post::get_post_store(db),
        }
    }
}
