use crate::ctx::Ctx;
use crate::model::task::{Task, TaskBackendModelController, TaskFilter, TaskForCreate, TaskForUpdate};
use crate::model::ModelManager;
use crate::web::rpc::params::{ParamsById, ParamsForCreate, ParamsForUpdate, ParamsList};
use crate::web::Result;

pub async fn create_task(
    ctx: Ctx,
    mm: ModelManager,
    params: ParamsForCreate<TaskForCreate>,
) -> Result<Task> {
    let ParamsForCreate { data } = params;

    let id = TaskBackendModelController::create(&ctx, &mm, data).await?;
    let task =  TaskBackendModelController::get(&ctx, &mm, id).await?;
    
    Ok(task)
}

pub async fn list_tasks(
    ctx: Ctx,
    mm: ModelManager,
    params: ParamsList<TaskFilter>,
) -> Result<Vec<Task>> {
    let tasks = TaskBackendModelController::list(&ctx, &mm, params.filters, params.list_options).await?;
    
    Ok(tasks)
}

pub async fn update_task(
    ctx: Ctx,
    mm: ModelManager,
    params: ParamsForUpdate<TaskForUpdate>,
) -> Result<Task> {
    let ParamsForUpdate { id, data } = params;

    TaskBackendModelController::update(&ctx, &mm, id, data);
    let task =  TaskBackendModelController::get(&ctx, &mm, id).await?;

    Ok(task)
}

pub async fn delete_task(
    ctx: Ctx,
    mm: ModelManager,
    params: ParamsById,
) -> Result<Task> {
    let ParamsById { id } = params;

    let task = TaskBackendModelController::get(&ctx, &mm, id).await?;
    TaskBackendModelController::delete(&ctx, &mm, id).await?;
    
    Ok(task)
}
