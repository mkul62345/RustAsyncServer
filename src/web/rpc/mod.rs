use std::sync::Arc;
use crate::web::mw_auth::CtxW;
use axum::{body, extract::State, response::{IntoResponse, Response}, routing::post, Json, Router};
use serde::Deserialize;
use serde_json::{from_value, json, to_value, Value};
use task_rpc::delete_task;
use tracing::debug;

use crate::web::rpc::task_rpc::{list_tasks, create_task};
use crate::{ctx::Ctx, model::ModelManager, web::{Error, Result}};

mod task_rpc;

// region: RPC Types
#[derive(Debug)]
pub struct RpcInfo {
    pub id: Option<Value>,
    pub method: String,
}

#[derive(Deserialize)]
struct RpcRequest {
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

#[derive(Deserialize)]
pub struct ParamsForCreate<D> {
    data: D,
}

#[derive(Deserialize)]
pub struct ParamsForUpdate<D> {
    id: i64,
    data: D,
}

#[derive(Deserialize)]
pub struct ParamsById {
    id: i64,
}
// endregion: RPC Types

async fn rpc_handler(
    State(mm): State<ModelManager>,
    ctx: CtxW,
    Json(rpc_req): Json<RpcRequest>
) -> Response {
    let ctx = ctx.0;

    let rpc_info = RpcInfo {
        id: rpc_req.id.clone(),
        method: rpc_req.method.clone(),
    };

    // Execute and Store RpcInfo in response
    let mut res = _rpc_handler(ctx, mm, rpc_req).await.into_response();
    res.extensions_mut().insert(Arc::new(rpc_info));

    res
}

macro_rules! exec_rpc_fn {
    // Without params
    ($rpc_fn:expr, $ctx:expr, $mm:expr) => {
        $rpc_fn($ctx, $mm).await.map(to_value)??
    };

     // With params
    ($rpc_fn:expr, $ctx:expr, $mm:expr, $rpc_params:expr) => {{  
        let rpc_fn_name = stringify!($rpc_fn);
        let params = $rpc_params.ok_or(Error::RpcMissingParams { 
                rpc_mathod: rpc_fn_name.to_string(),
            })?;

        let params =
            from_value(params).map_err(|_| Error::RpcFailJsonParams {
                rpc_mathod: rpc_fn_name.to_string(),
            })?;

        $rpc_fn($ctx, $mm, params).await.map(to_value)??
    }};

}

async fn _rpc_handler(
    ctx: Ctx,
    mm: ModelManager,
    rpc_req: RpcRequest,
) -> Result<Json<Value>> {
    let RpcRequest { 
        id: rpc_id,
        method: rpc_method,
        params: rpc_params, 
    } = rpc_req;

    debug!("_rpc_handler - method: {rpc_method}");
    

    let result_json: Value = match rpc_method.as_str() {
        // Task RPC methods.
        "create_task" => exec_rpc_fn!(create_task, ctx, mm, rpc_params),

        "list_tasks" => exec_rpc_fn!(list_tasks, ctx, mm),

        //TODO: "update_task" => todo!(),

        "delete_task" => exec_rpc_fn!(delete_task, ctx, mm, rpc_params),

        // Fallback Error
        _ => return Err(Error::RpcMethodUnknown(rpc_method)),
    };

    let body_response = json!({
        "id": rpc_id,
        "result": result_json
    });
    
    Ok(Json(body_response))
}

pub fn routes(mm: ModelManager) -> Router {
    Router::new()
        .route("/rpc", post(rpc_handler))
        .with_state(mm)
}