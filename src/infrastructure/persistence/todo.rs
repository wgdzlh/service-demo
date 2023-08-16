use std::sync::{Arc, Mutex};

use crate::{
    app::log::*,
    entity::Todo,
    repository::{Error, Result, TodoQuery, TodoRepo, TodoUpdate},
};

/// In-memory Todo store
pub type TodoStore = Arc<dyn TodoRepo + Send + Sync>;

pub(super) fn get_todo_store() -> TodoStore {
    Arc::new(TodoRepoImp {
        data: Mutex::default(),
    })
}

struct TodoRepoImp {
    data: Mutex<Vec<Todo>>,
}

impl TodoRepo for TodoRepoImp {
    fn create(&self, mut item: Todo) -> Result<i32> {
        info!(?item, "create todo");
        let mut list = self.data.lock()?;
        let max_id = match list.iter().max_by_key(|x| x.id) {
            Some(x) => x.id + 1,
            None => 1,
        };
        item.id = max_id;
        list.push(item);
        Ok(max_id)
    }

    fn update(&self, item: TodoUpdate) -> Result<()> {
        info!(?item, "update todo");
        if item.id <= 0 {
            return Err(Error::IdNotFound { id: item.id });
        }
        if item.value.is_none() && item.done.is_none() {
            return Ok(());
        }
        self.data
            .lock()?
            .iter_mut()
            .find(|x| x.id == item.id)
            .map(|x| {
                if let Some(v) = item.value {
                    x.value = v
                }
                if let Some(v) = item.done {
                    x.done = v
                }
                Ok(())
            })
            .unwrap_or(Err(Error::IdNotFound { id: item.id }))
    }

    fn delete(&self, ids: Vec<i32>) -> Result<()> {
        info!(?ids, "delete todos");
        self.data.lock()?.retain(|x| !ids.contains(&x.id));
        Ok(())
    }

    fn fetch(&self, id: i32) -> Result<Todo> {
        info!(?id, "fetch todo");
        if id <= 0 {
            return Err(Error::IdNotFound { id });
        }
        let list = self.data.lock()?;
        let target = list.iter().find(|x| x.id == id);
        match target {
            Some(x) => Ok(x.clone()),
            None => Err(Error::IdNotFound { id }),
        }
    }

    fn query(&self, mut req: TodoQuery) -> Result<Vec<Todo>> {
        info!(?req, "query todos");
        if let Some(v) = req.value.as_mut() {
            v.make_ascii_lowercase();
        }
        Ok(self
            .data
            .lock()?
            .iter()
            .filter(|x| {
                if let Some(ref v) = req.value {
                    if !x.value.to_lowercase().contains(v) {
                        return false;
                    }
                }
                if let Some(v) = req.done {
                    if x.done != v {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect())
    }
}
