use axum::extract::{State, Path, Json, FromRef};
use axum::routing::{Route, get, post, delete};
use axum::Router;
use crate::ctx::Ctx;
use crate::{Error, Result};
use crate::model::{ModelController, Ticket, TicketForCreate};


#[derive(Clone, FromRef)]
struct AppState {
    mc: ModelController
}

pub fn routes(mc: ModelController) -> Router {
    let app_state = AppState { mc };
    Router::new()
        .route("/tickets", post(create_ticket).get(list_tickets))
        .route("/tickets/{:id}", delete(delete_ticket))
        .with_state(app_state)
}

// region: REST Handlers
async fn create_ticket(
    State(mc): State<ModelController>,
    ctx: Ctx,
    Json(ticket_fc): Json<TicketForCreate>,
) -> Result<Json<Ticket>>{

    let ticket = mc.create_ticket(ctx, ticket_fc).await?;

    Ok(Json(ticket))
}

//   TODO: add filtration functionality
async fn list_tickets(
    State(mc): State<ModelController>,
    ctx: Ctx,
) -> Result<Json<Vec<Ticket>>>{

    let tickets = mc.list_tickets(ctx).await?;
    Ok(Json(tickets))
}

async fn delete_ticket(
    State(mc): State<ModelController>,
    ctx: Ctx,
    Path(id): Path<u64>,
) -> Result<Json<Ticket>>{

    let ticket = mc.delete_ticket(ctx, id).await?;
    Ok(Json(ticket))
}
// endregion: REST Handlers