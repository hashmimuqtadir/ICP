use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Principal(String);

fn current_time() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Role {
    Admin,
    Organizer,
    User,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Error {
    NotFound,
    AlreadyExists,
    NotAuthorized,
    InvalidOperation,
    InsufficientFunds,
    SoldOut,
    LimitExceeded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub event_id: u64,
    pub name: String,
    pub date: u64, // UNIX timestamp
    pub venue: String,
    pub price: u64,
    pub total_tickets: u64,
    pub available_tickets: u64,
    pub organizer: Principal,
    pub is_active: bool,
    pub description: String,
    pub image_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketTransfer {
    pub from: Principal,
    pub to: Principal,
    pub price: u64,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketMetadata {
    pub event_name: String,
    pub ticket_class: String,
    pub seat_info: Option<String>,
    pub purchase_date: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticket {
    pub token_id: u64,
    pub event_id: u64,
    pub owner: Principal,
    pub original_price: u64,
    pub current_price: u64,
    pub purchase_history: Vec<TicketTransfer>,
    pub is_valid: bool,
    pub metadata: Option<TicketMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenMetadata {
    pub token_id: u64,
    pub owner: Principal,
    pub metadata_blob: Option<Vec<u8>>, // Blob of metadata, e.g. JSON or CBOR
    pub properties: Vec<(String, String)>,
    pub is_approved: bool,
}

pub struct TicketingSystem {
    next_event_id: u64,
    next_ticket_id: u64,
    events: HashMap<u64, Event>,
    tickets: HashMap<u64, Ticket>,
    user_roles: HashMap<Principal, Role>,
    user_tickets: HashMap<Principal, Vec<u64>>,
    organizer_events: HashMap<Principal, Vec<u64>>,
    owner: Principal,
    // Constants
    max_resale_multiplier: f64,
    platform_fee_percentage: u8, // percent, e.g., 2%
}

impl TicketingSystem {
    pub fn new(owner: Principal) -> Self {
        let mut user_roles = HashMap::new();
        user_roles.insert(owner.clone(), Role::Admin);

        TicketingSystem {
            next_event_id: 1,
            next_ticket_id: 1,
            events: HashMap::new(),
            tickets: HashMap::new(),
            user_roles,
            user_tickets: HashMap::new(),
            organizer_events: HashMap::new(),
            owner,
            max_resale_multiplier: 1.2,
            platform_fee_percentage: 2,
        }
    }

    // EVENT MANAGEMENT: (continued from previous)
    // Assume create_event, get_event, get_all_events, get_active_events are implemented as before.

    pub fn update_event(
        &mut self,
        caller: &Principal,
        event_id: u64,
        name: Option<String>,
        date: Option<u64>,
        venue: Option<String>,
        price: Option<u64>,
        total_tickets: Option<u64>,
        is_active: Option<bool>,
        description: Option<String>,
        image_url: Option<Option<String>>,
    ) -> Result<Event, Error> {
        let event = self.events.get_mut(&event_id).ok_or(Error::NotFound)?;

        // Authorization check
        let caller_role = self.user_roles.get(caller).unwrap_or(&Role::User);
        if &event.organizer != caller && caller_role != &Role::Admin {
            return Err(Error::NotAuthorized);
        }

        // Update fields safely
        if let Some(new_name) = name {
            if new_name.is_empty() {
                return Err(Error::InvalidOperation);
            }
            event.name = new_name;
        }

        if let Some(new_date) = date {
            if new_date < current_time() {
                return Err(Error::InvalidOperation);
            }
            event.date = new_date;
        }

        if let Some(new_venue) = venue {
            event.venue = new_venue;
        }

        if let Some(new_price) = price {
            event.price = new_price;
        }

        if let Some(new_total) = total_tickets {
            if new_total < event.total_tickets {
                // Don't decrease total tickets below previous total (could complicate sold tickets)
                return Err(Error::InvalidOperation);
            } else {
                let added_tickets = new_total - event.total_tickets;
                event.available_tickets += added_tickets;
                event.total_tickets = new_total;
            }
        }

        if let Some(active) = is_active {
            event.is_active = active;
        }

        if let Some(new_desc) = description {
            event.description = new_desc;
        }

        if let Some(new_img_url) = image_url {
            event.image_url = new_img_url;
        }

        Ok(event.clone())
    }

    pub fn cancel_event(&mut self, caller: &Principal, event_id: u64) -> Result<bool, Error> {
        let event = self.events.get_mut(&event_id).ok_or(Error::NotFound)?;

        let caller_role = self.user_roles.get(caller).unwrap_or(&Role::User);
        if &event.organizer != caller && caller_role != &Role::Admin {
            return Err(Error::NotAuthorized);
        }

        event.is_active = false;

        // TODO: Implement refund logic for purchased tickets

        Ok(true)
    }

    // TICKET MANAGEMENT

    pub fn purchase_ticket(&mut self, caller: &Principal, event_id: u64) -> Result<Ticket, Error> {
        let event = self.events.get_mut(&event_id).ok_or(Error::NotFound)?;

        if !event.is_active || event.available_tickets == 0 || event.date < current_time() {
            if !event.is_active {
                return Err(Error::InvalidOperation);
            }
            if event.available_tickets == 0 {
                return Err(Error::SoldOut);
            }
            return Err(Error::InvalidOperation);
        }

        // Payment processing skipped here

        // Create transfer record
        let transfer = TicketTransfer {
            from: event.organizer.clone(),
            to: caller.clone(),
            price: event.price,
            timestamp: current_time(),
        };

        // Create Ticket Metadata
        let metadata = TicketMetadata {
            event_name: event.name.clone(),
            ticket_class: "Standard".to_string(),
            seat_info: None,
            purchase_date: current_time(),
        };

        let token_id = self.next_ticket_id;
        self.next_ticket_id += 1;

        let ticket = Ticket {
            token_id,
            event_id,
            owner: caller.clone(),
            original_price: event.price,
            current_price: event.price,
            purchase_history: vec![transfer],
            is_valid: true,
            metadata: Some(metadata),
        };

        self.tickets.insert(token_id, ticket.clone());

        event.available_tickets -= 1;

        self.user_tickets.entry(caller.clone())
            .or_default()
            .push(token_id);

        Ok(ticket)
    }

    pub fn get_ticket(&self, token_id: u64) -> Result<Ticket, Error> {
        self.tickets.get(&token_id).cloned().ok_or(Error::NotFound)
    }

    pub fn verify_ticket(&self, token_id: u64, owner: &Principal) -> Result<bool, Error> {
        let ticket = self.tickets.get(&token_id).ok_or(Error::NotFound)?;
        Ok(ticket.is_valid && &ticket.owner == owner)
    }

    pub fn get_event_tickets(&self, event_id: u64) -> Vec<Ticket> {
        self.tickets.values()
            .filter(|ticket| ticket.event_id == event_id)
            .cloned()
            .collect()
    }

    pub fn get_user_tickets(&self, user: &Principal) -> Vec<Ticket> {
        if let Some(token_ids) = self.user_tickets.get(user) {
            token_ids.iter().filter_map(|tid| self.tickets.get(tid)).cloned().collect()
        } else {
            Vec::new()
        }
    }

    pub fn list_ticket_for_resale(
        &mut self,
        caller: &Principal,
        token_id: u64,
        price: u64,
    ) -> Result<Ticket, Error> {
        let ticket = self.tickets.get_mut(&token_id).ok_or(Error::NotFound)?;

        if &ticket.owner != caller {
            return Err(Error::NotAuthorized);
        }

        let max_price = (ticket.original_price as f64 * self.max_resale_multiplier).floor() as u64;

        if price > max_price {
            return Err(Error::LimitExceeded);
        }

        ticket.current_price = price;
        Ok(ticket.clone())
    }

    pub fn buy_resale_ticket(&mut self, caller: &Principal, token_id: u64) -> Result<Ticket, Error> {
        let ticket = self.tickets.get(&token_id).ok_or(Error::NotFound)?;

        if &ticket.owner == caller {
            return Err(Error::InvalidOperation);
        }

        let event = self.events.get(&ticket.event_id).ok_or(Error::NotFound)?;

        if !event.is_active || event.date < current_time() {
            return Err(Error::InvalidOperation);
        }

        // Payment skipped

        // Transfer ticket ownership
        let transfer = TicketTransfer {
            from: ticket.owner.clone(),
            to: caller.clone(),
            price: ticket.current_price,
            timestamp: current_time(),
        };

        let mut ticket = ticket.clone();
        ticket.purchase_history.push(transfer);
        ticket.owner = caller.clone();

        self.tickets.insert(token_id, ticket.clone());

        // Update user tickets collections

        // Remove token from previous owner
        if let Some(tokens) = self.user_tickets.get_mut(&ticket.purchase_history[ticket.purchase_history.len() - 2].to) {
            tokens.retain(|&tid| tid != token_id);
        }

        // Add token to new owner
        self.user_tickets.entry(caller.clone())
            .or_default()
            .push(token_id);

        Ok(ticket)
    }

    pub fn transfer_ticket(
        &mut self,
        caller: &Principal,
        token_id: u64,
        recipient: Principal,
    ) -> Result<Ticket, Error> {
        let ticket = self.tickets.get(&token_id).ok_or(Error::NotFound)?;

        if &ticket.owner != caller {
            return Err(Error::NotAuthorized);
        }

        if &recipient == caller {
            return Err(Error::InvalidOperation);
        }

        let transfer = TicketTransfer {
            from: caller.clone(),
            to: recipient.clone(),
            price: 0,
            timestamp: current_time(),
        };

        let mut ticket = ticket.clone();
        ticket.purchase_history.push(transfer);
        ticket.owner = recipient.clone();

        self.tickets.insert(token_id, ticket.clone());

        // Update user tickets maps
        if let Some(tokens) = self.user_tickets.get_mut(caller) {
            tokens.retain(|&tid| tid != token_id);
        }
        self.user_tickets.entry(recipient)
            .or_default()
            .push(token_id);

        Ok(ticket)
    }

    pub fn invalidate_ticket(&mut self, caller: &Principal, token_id: u64) -> Result<Ticket, Error> {
        let ticket = self.tickets.get(&token_id).ok_or(Error::NotFound)?;
        let event = self.events.get(&ticket.event_id).ok_or(Error::NotFound)?;

        if caller != &event.organizer && caller != &self.owner {
            return Err(Error::NotAuthorized);
        }

        let mut ticket = ticket.clone();
        ticket.is_valid = false;

        self.tickets.insert(token_id, ticket.clone());

        Ok(ticket)
    }

    // ROLE MANAGEMENT

    pub fn assign_role(&mut self, caller: &Principal, user: Principal, role: Role) -> Result<bool, Error> {
        let caller_role = self.user_roles.get(caller).unwrap_or(&Role::User);

        if caller_role != &Role::Admin {
            return Err(Error::NotAuthorized);
        }

        self.user_roles.insert(user, role);

        Ok(true)
    }

    pub fn get_user_role(&self, user: &Principal) -> Role {
        self.user_roles.get(user).cloned().unwrap_or(Role::User)
    }

    // DIP721 NFT IMPLEMENTATION

    pub fn get_token_metadata(&self, token_id: u64) -> Option<TokenMetadata> {
        let ticket = self.tickets.get(&token_id)?;

        let properties = vec![
            ("eventId".to_string(), ticket.event_id.to_string()),
            ("isValid".to_string(), ticket.is_valid.to_string()),
        ];

        Some(TokenMetadata {
            token_id,
            owner: ticket.owner.clone(),
            metadata_blob: None,
            properties,
            is_approved: false,
        })
    }

    pub fn total_supply(&self) -> u64 {
        self.next_ticket_id.saturating_sub(1)
    }

    pub fn balance_of(&self, owner: &Principal) -> u64 {
        self.user_tickets.get(owner).map(|v| v.len() as u64).unwrap_or(0)
    }

    pub fn owner_of(&self, token_id: u64) -> Option<Principal> {
        self.tickets.get(&token_id).map(|ticket| ticket.owner.clone())
    }

    pub fn tokens_of(&self, owner: &Principal) -> Vec<u64> {
        self.user_tickets.get(owner).cloned().unwrap_or_default()
    }

    // ORGANIZER FUNCTIONS

    pub fn get_organizer_events(&self, organizer: &Principal) -> Vec<Event> {
        if let Some(event_ids) = self.organizer_events.get(organizer) {
            event_ids.iter()
                .filter_map(|id| self.events.get(id).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn get_event_stats(&self, caller: &Principal, event_id: u64) -> Result<(u64, u64, u64), Error> {
        let event = self.events.get(&event_id).ok_or(Error::NotFound)?;

        let caller_role = self.user_roles.get(caller).unwrap_or(&Role::User);

        if &event.organizer != caller && caller_role != &Role::Admin {
            return Err(Error::NotAuthorized);
        }

        let mut total_sold = 0;
        let mut total_revenue = 0;
        let mut valid_tickets = 0;

        for ticket in self.tickets.values() {
            if ticket.event_id == event_id {
                total_sold += 1;
                total_revenue += ticket.original_price;
                if ticket.is_valid {
                    valid_tickets += 1;
                }
            }
        }

        Ok((total_sold, total_revenue, valid_tickets))
    }
}

