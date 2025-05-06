use crate::{Error, Result};
use serde::{Serialize, Deserialize};
use std::sync::{Arc, Mutex};



// region: Tickets
#[derive(Clone, Debug, Serialize)]
pub struct Ticket{
    pub id: u64,
    pub title: String,
}

#[derive(Deserialize)]
pub struct TicketForCreate {
    pub title: String,
}

// endregion : Tickets

// region: Model Controller
#[derive(Clone)]
pub struct ModelController {
    // TODO:  change into proper DB connection / sqlx
    tickets_storage: Arc<Mutex<Vec<Option<Ticket>>>>,
}

//Constructor
impl ModelController {
    pub async fn new() -> Result<Self> {
        Ok(Self { tickets_storage: Arc::default() })
    }
    
}


// CRUD Impl
impl ModelController{
    pub async fn create_ticket(&self, ticket_fc: TicketForCreate) -> Result<Ticket>{
        let mut store = self.tickets_storage.lock().unwrap();

        let id = store.len() as u64; //Sequentially increment the id by 1. FIXME: Predictable ID : OWASP api vulnerability
        let ticket = Ticket {
            id,
            title: ticket_fc.title,
        };
        store.push(Some(ticket.clone()));

        Ok(ticket)
    }

    pub async fn list_tickets(&self) -> Result<Vec<Ticket>>{
        let store = self.tickets_storage.lock().unwrap();

        let tickets = store
            .iter()
            .filter_map(|t| t.clone())
            .collect();

        Ok(tickets)
    }

    pub async fn delete_ticket(&self, id:u64) ->Result<Ticket>{
        let mut store = self.tickets_storage.lock().unwrap();

        let ticket = store.get_mut(id as usize).and_then(|t| t.take());

        ticket.ok_or(Error::TicketDeleteFailIdNotFound { id })
    }

}


// endregion : Model Controller