use std::error::Error;

use serenity::async_trait;

use crate::traits::wishlist_db::WishlistDB;


struct MockWishlistDB {}

#[async_trait]
impl WishlistDB for MockWishlistDB {
    async fn get_users_with_series_card<'a> (
        &'a self, 
        cards: Vec<(&'a str, &'a str)>
    ) -> Result<Vec<((&str, &str), Vec<String>)>, Box<dyn Error + Send + Sync>> {
        todo!()
    }

    async fn get_users_with_series<'a>(
        &'a self, 
        series:&Vec<&'a str>
    ) -> Result<Vec<(&str, Vec<(String, i32)>)>, Box<dyn Error + Send + Sync>> {
        todo!()
    }
    
    async fn add_all_to_wishlist(
        &self, 
        user_id:&str, 
        series:&str, 
        card_names:Vec<&str>
    ) -> Result<i32, Box<dyn Error + Send + Sync>> {
        todo!()
    }
    
    async fn remove_all_from_wishlist(
        &self, 
        user_id:&str, 
        series:&str, 
        card_names:Vec<&str>
    ) -> Result<(i32, i32), Box<dyn Error + Send + Sync>> {
        todo!()
    }
    
    async fn get_user_wishlisted_series(
        &self, 
        user_id: &str
    ) -> Vec<String> {
        todo!()
    }
    
    async fn get_user_wishlisted_cards_count(
        &self, 
        user_id: &str, 
        series: &str
    ) -> i32 {
        todo!()
    }
    
    async fn get_user_wishlisted_cards(
        &self, 
        user_id: &str, 
        series: &str
    ) -> Vec<String> {
        todo!()
    }
    
    async fn user_has_card(
        &self, 
        user_id: &str, 
        series: &str, 
        card: &str
    ) -> bool {
        todo!()
    }
    
    async fn remove_series_from_wishlist(
        &self, 
        user_id:&str, 
        series:&str
    ) -> Result<i32, Box<dyn Error + Send + Sync>> {
        todo!()
    }
}