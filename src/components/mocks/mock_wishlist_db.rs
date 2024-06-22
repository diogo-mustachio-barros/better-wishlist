use std::error::Error;
use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::sync::RwLock;
use serenity::async_trait;

use crate::traits::wishlist_db::WishlistDB;

#[derive(Debug, Clone)]
struct MockWishlistDBError{
    description: String
}

impl Display for MockWishlistDBError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description)
    }
}

impl Error for MockWishlistDBError {}

struct MockWishlistDB {
    wishlists: RwLock<HashMap<String, HashMap<String, HashSet<String>>>>
}

#[async_trait]
impl WishlistDB for MockWishlistDB {
    
    async fn get_users_with_series_card<'b> (
        &'b self, 
        cards: Vec<(&'b str, &'b str)>
    ) -> Result<Vec<((&str, &str), Vec<String>)>, Box<dyn Error + Send + Sync>> 
    {
        let res = cards.iter().map(|(series, card)| {
            let users = self.wishlists.read().unwrap().iter()
                .filter( |(_, v)| 
                    v.get(&series.to_string()).is_some_and(|cards| cards.contains(&card.to_string()))
                )
                .map(|(k, _)| k.clone().to_string())
                .collect();

            ((series.to_owned(), card.to_owned()), users)
        }).collect::<Vec<((&str, &str), Vec<String>)>>();

        Ok(res)
    }

    async fn get_users_with_series<'b> (
        &'b self, 
        series:&Vec<&'b str>
    ) -> Result<Vec<(&str, Vec<(String, i32)>)>, Box<dyn Error + Send + Sync>> {
        let res = series.iter().map(|series_name| {
            let users = self.wishlists.read().unwrap().iter()
                .filter( |(_, v)| 
                    v.contains_key(&series_name.to_string())
                )
                .map(|(k, v)| 
                    (k.clone().to_string(), v.len().try_into().unwrap())
                )
                .collect();

            (series_name.to_owned(), users)
        }).collect::<Vec<(&str, Vec<(String, i32)>)>>();

        Ok(res)
    }
    
    async fn add_all_to_wishlist<'b> (
        &self, 
        user_id: &'b str, 
        series: &'b str, 
        card_names: Vec<&'b str>
    ) -> Result<i32, Box<dyn Error + Send + Sync>>
    {
        let prev_count = self.get_user_wishlisted_cards_count(user_id,series).await;

        let series_s = series.to_string();
        let user_id_s = user_id.to_string();
        let card_names_s: Vec<String> = 
            card_names.clone().iter().map(|s| s.to_string()).collect();

        {
            let mut wishlists = self.wishlists.write().unwrap();
            match wishlists.get_mut(&user_id_s) {
                Some(user_wishlist) => {
                    match user_wishlist.get_mut(&series_s) {
                        Some(wishlisted_cards) => {
                            wishlisted_cards.extend(card_names_s);
                        }
                        None => {
                            let cards_set = card_names_s.iter().cloned().collect();
                            user_wishlist.insert(series_s, cards_set);
                        }
                    }
                }
                None => {
                    let cards_set = card_names_s.iter().cloned().collect();
                    let mut series_wishlist = HashMap::new();
                    series_wishlist.insert(series_s, cards_set);
                    
                    wishlists.insert(user_id_s, series_wishlist);
                }
            }
        }

        let curr_count = self.get_user_wishlisted_cards_count(user_id,series).await;
        Ok(curr_count - prev_count)
    }
    
    async fn remove_all_from_wishlist(
        &self, 
        user_id:&str, 
        series:&str, 
        card_names:Vec<&str>
    ) -> Result<(i32, i32), Box<dyn Error + Send + Sync>> {
        let res = self.wishlists.write().unwrap().get_mut(user_id)
            .and_then( |user_wishlist|
                user_wishlist.get_mut(series)
            )
            .map( |wishlisted_cards| {
                let mut removed_count = 0;
                for card in card_names {
                    if wishlisted_cards.remove(card) {
                        removed_count += 1;
                    }
                }

                (removed_count, wishlisted_cards.len().try_into().unwrap())
            }).unwrap_or((0, 0));

        Ok(res)
    }
    
    async fn get_user_wishlisted_series(
        &self, 
        user_id: &str
    ) -> Vec<String> {
        self.wishlists.read().unwrap().get(user_id)
            .map(|user_wishlist|
                user_wishlist.keys().clone()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>()
            )
            .unwrap_or(vec![])
    }
    
    async fn get_user_wishlisted_cards_count(
        &self, 
        user_id: &str, 
        series: &str
    ) -> i32 {
        self.get_user_wishlisted_cards(user_id, series).await
            .len().try_into().unwrap()
    }
    
    async fn get_user_wishlisted_cards(
        &self, 
        user_id: &str, 
        series: &str
    ) -> Vec<String> {
        self.wishlists.read().unwrap().get(user_id)
            .and_then( |user_wishlist|
                user_wishlist.get(series)
            )
            .map( |wishlisted_cards|
                wishlisted_cards.clone().iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>()
            )
            .unwrap_or(vec![])
    }
    
    async fn user_has_card(
        &self, 
        user_id: &str, 
        series: &str, 
        card: &str
    ) -> bool {
        self.get_user_wishlisted_cards(user_id, series).await
            .contains(&card.to_string())
    }
    
    async fn remove_series_from_wishlist (
        &self,
        user_id:&str, 
        series:&str
    ) -> Result<i32, Box<dyn Error + Send + Sync>> {
        let res = self.wishlists.write().unwrap().get_mut(user_id)
            .and_then(|user_wishlist|
                user_wishlist.remove(series)
            )
            .and_then(|wishlisted_cards|{
                let count:i32 = wishlisted_cards.len().try_into().unwrap();
                Some(count)
            }).unwrap_or(0);
        
        Ok(res)
    }
}