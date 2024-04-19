use serenity::all::UserId;
use std::{cell::RefCell, collections::HashMap, sync::{Arc, RwLock}};

enum WishlistDisplay {
    Series,
    Cards
}

pub struct WishlistInteraction {
    // discord info
    user_id: UserId,


    // business info
    listing_type: WishlistDisplay,
    curr_page: i32,
    series_count: i32,
    cards_count: i32
}

impl WishlistInteraction {
    pub fn new(user_id: UserId) -> WishlistInteraction {
        WishlistInteraction { 
            user_id, 
            listing_type: WishlistDisplay::Series, 
            curr_page: 0, 
            series_count: -1, 
            cards_count: -1 
        }
    }
}




pub struct InteractionManager {
    pending_interactions: Arc<RwLock<HashMap<UserId, WishlistInteraction>>>
}

impl InteractionManager {
    
    pub fn new()-> InteractionManager {
        InteractionManager {
            pending_interactions: Arc::new(RwLock::new(HashMap::new()))
        }
    }
    
    pub fn add_interaction(&self, user_id: UserId) {
    //     if self.pending_interactions.get_mut().unwrap().contains_key(&&user_id) {
    //         // delete previous interaction
    //         self.pending_interactions.get_mut().unwrap().remove(&user_id);
    //     }

    //     // add new interaction
    //     self.pending_interactions.get_mut().unwrap().insert(user_id, WishlistInteraction::new(user_id));
    }
}