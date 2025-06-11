use crate::{ctx::Ctx, ModelManager};
use crate::model::{Error, Result};
use modql::field::Fields;
use modql::filter::{FilterNodes, ListOptions, OpValsBool, OpValsInt64, OpValsString};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use super::base::{self, DbBackendModelController};

// region: Task BackendModelController (Bmc)
pub struct TaskBackendModelController;

impl DbBackendModelController for TaskBackendModelController {
    const TABLE: &'static str = "task";
}

impl TaskBackendModelController{
    pub async fn create(
        _ctx: &Ctx,
        mm: &ModelManager,
        task_create: TaskForCreate,
    ) -> Result<i64> {
        base::create::<Self, _>(_ctx, mm, task_create).await
    }

    pub async fn get(
        ctx: &Ctx,
        mm: &ModelManager,
        id: i64,
    ) -> Result<Task> {
        base::get::<Self, _>(ctx, mm, id).await
        }

    pub async fn list(
        ctx: &Ctx,
        mm: &ModelManager,
        filters: Option<Vec<TaskFilter>>,
        list_options: Option<ListOptions>,
    ) -> Result<Vec<Task>> {
        base::list::<Self, _, _>(ctx, mm, filters, list_options).await
    }
    
    pub async fn update(
        ctx: &Ctx,
        mm: &ModelManager,
        id: i64,
        task_update: TaskForUpdate,
    ) -> Result<()> {
        base::update::<Self, _>(ctx, mm, id, task_update).await
    }

    pub async fn delete(
        ctx: &Ctx,
        mm: &ModelManager,
        id: i64,
    ) -> Result<()> {
        base::delete::<Self>(ctx, mm, id).await
    }
    }
// endregion: Task BackendModelController (Bmc)

// region: Task types
// Split tasks into subtypes based on intent
// To be sent to the model layer
#[derive(Debug, Clone, FromRow, Serialize, Fields)]
pub struct Task {
    pub id: i64,
    
    pub title: String,
    pub done: bool, 
}

#[derive(Deserialize, Fields)]
pub struct TaskForCreate {
    pub title: String,
}

#[derive(Deserialize, Fields, Default)]
pub struct TaskForUpdate {
    pub title: Option<String>,
    pub done: Option<bool>,
}

#[derive(FilterNodes, Deserialize, Default, Debug)]
pub struct TaskFilter {
    id: Option<OpValsInt64>,

    title: Option<OpValsString>,
    done: Option<OpValsBool>,
}
// endregion: Task types

// region: Tests
#[cfg(test)]
mod tests {
    #![allow(unused)]
    use serde_json::json;
    use serial_test::serial;
    use crate::{_dev_utils, model::task};
    use super::*;
    //use anyhow::Result;

    #[serial]
    #[tokio::test]
    async fn test_create_ok() -> Result<()>{
        // Setup
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_ctx();
        let fx_title = "test_create_ok title";

        // Execute
        let task_create = TaskForCreate {
            title: fx_title.to_string(),
        };
        let id = TaskBackendModelController::create(&ctx, &mm, task_create).await?;

        // Validate
        let task = TaskBackendModelController::get(&ctx, &mm, id).await?;
        assert_eq!(task.title, fx_title);

        // Cleanup
        TaskBackendModelController::delete(&ctx, &mm, id).await?;

        Ok(())
    }

    #[serial]
    #[tokio::test]
    async fn test_list_all_ok() -> Result<()>{
        // Setup
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_ctx();
        let fx_titles = &["test_list_all_ok 01", "test_list_all_ok 02"];
        _dev_utils::seed_tasks(&ctx, &mm, fx_titles).await?;

        // Execute
        let tasks = TaskBackendModelController::list(&ctx, &mm, None, None).await?;

        // Validate
        let tasks: Vec<Task> = tasks
            .into_iter()
            .filter(|t| t.title.starts_with("test_list_all_ok"))
            .collect();

        assert_eq!(tasks.len(), 2 , "number of seeded tasks.");

        // Cleanup
        for task in tasks.iter() {
            TaskBackendModelController::delete(&ctx, &mm, task.id).await?;
        }

        Ok(())
    }    

    #[serial]
    #[tokio::test]
    async fn test_list_by_filter_ok() -> Result<()>{
        // Setup
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_ctx();
        let fx_titles = &[
            "test_list_by_filter_ok 01.a", 
            "test_list_by_filter_ok 01.b",
            "test_list_by_filter_ok 02.a",
            "test_list_by_filter_ok 02.b",
            "test_list_by_filter_ok 03",
            ];
        _dev_utils::seed_tasks(&ctx, &mm, fx_titles).await?;

        // Execute
        let filters: Vec<TaskFilter> = serde_json::from_value(json!([
            {
            "title": {
                "$endsWith": ".a",
                "$containsAny": ["01", "02"],
                }
            },
            {
            "title": {
                "$contains": "03",
            }
        }]))?;

        let list_options = serde_json::from_value(json!({
            "order_bys": "!id"
        }))?;

        let tasks = TaskBackendModelController::list(&ctx, &mm, Some(filters), list_options).await?;

        // Validate
        assert_eq!(tasks.len(), 3);
        assert!(tasks[0].title.ends_with("03"));
        assert!(tasks[1].title.ends_with("02.a"));
        assert!(tasks[2].title.ends_with("01.a"));

        // Cleanup
        let tasks = TaskBackendModelController::list(
            &ctx,
            &mm,
            Some(serde_json::from_value(json!([{
                "title": {"$startsWith": "test_list_by_filter_ok"}
            }]))?),
            None
        )
        .await?; 
        assert_eq!(tasks.len(), 5);

        for task in tasks.iter() {
            TaskBackendModelController::delete(&ctx, &mm, task.id).await?;
        }

        Ok(())
    }    

    
    #[serial]
    #[tokio::test]
    async fn test_get_err_not_found() -> Result<()>{
        // Setup
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_ctx();
        let fx_id = 100; //100 cannot be valid due to the schema

        // Execute
        let res = TaskBackendModelController::get(&ctx, &mm, fx_id).await;

        // Validate
        assert!(
            matches!(
                res,
                Err(Error::EntityNotFound { 
                    entity: "task",
                     id: 100 
                    })
            ),
            "EntityNotFound not matching"
        );

        Ok(())
    }

}
// endregion: Tests

