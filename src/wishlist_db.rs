use std::vec;

use mongodb::{self, bson::{doc, Bson, Document}, error::Error, options::FindOneAndUpdateOptions};
use serenity::futures::TryStreamExt;

use crate::util::series_to_search;

pub struct WishlistDB {
    db_client: mongodb::Client
}

pub async fn init_db(uri: impl AsRef<str>) -> Result<WishlistDB, Error> {
    // Create a new client and connect to the server
    let client = mongodb::Client::with_uri_str(uri).await;

    return client.map(|db_client| WishlistDB{db_client});
}

impl WishlistDB {
    // pub async fn add_to_wishlist(&self, user_id:&str, series:&str, card:&str) -> Option<Error> {
    //     self.add_all_to_wishlist(user_id, series, [card].to_vec()).await
    // }

    // pub async fn add_all_to_wishlist(&self, user_id:&str, series:&str, cards:Vec<&str>) -> Vec<(String, Error)>{
    //     let futures = FuturesUnordered::new();
    //     for card in cards {
    //         futures.push(self.add_to_wishlist(user_id, series, card))
    //     }
        
    //     let mut errors: Vec<(String, mongodb::error::Error)> = vec![];
    //     let results: Vec<Option<(String, mongodb::error::Error)>> = futures.collect().await;
    //     for res in results {
    //         match res {
    //             Some((card, err)) => errors.push((card, err)),
    //             None => ()
    //         }
    //     }

    //     return errors;
    // }

    pub async fn add_all_to_wishlist(&self, user_id:&str, series:&str, cards:Vec<&str>) -> Option<Error> {
        let collection = get_wishlist_collection(&self.db_client);
        
        let series_search = series_to_search(series);
        
        if !self.user_has_series(user_id, series).await {
            let res = collection.find_one_and_update(
                doc! {"id": user_id},
                doc! {"$addToSet": { "series": { "name": series, "search": &series_search, "cards": [] }}},
                FindOneAndUpdateOptions::builder()
                .upsert(true)
                .build()
            ).await;

            if res.is_err() {
                return Some(res.unwrap_err());
            }
        };

        let res = collection.find_one_and_update( 
            doc!{"id": user_id, "series.search": series_search}, 
            doc!{"$addToSet": { "series.$[elem].cards": doc!{"$each": cards} }}, 
            FindOneAndUpdateOptions::builder()
            .upsert(true)
            .array_filters(vec![doc! {"elem.name": series }])
            .build()
        ).await;

        match res {
            Ok(_) => None,
            Err(err) => Some(err)
        }
    }

    pub async fn get_wishlisted_users(&self, series:&str, card_name:&str) -> Result<Vec<String>, mongodb::error::Error> {
        let collection = get_wishlist_collection(&self.db_client);

        // println!("'{}'", card_name);
        let series_search = series_to_search(series);

        let res =
            collection.find(
                doc!{"series.search": series_search, "series.cards": card_name},
                None
            ).await;

        if res.is_err() {
            return Err(res.unwrap_err());
        }

        let cursor = res.unwrap();

        let ret = 
            cursor.try_collect().await
                  .unwrap_or_else(|_| vec![])
                  .iter()
                  .map(|doc| {
                    if let Some(bson) = doc.get("id") {
                        match bson {
                            Bson::String(user) => user.to_owned(),
                            _ => "".to_string()
                        }
                    } else {
                        "".to_string()
                    }
                  })
                  .collect();

        return Ok(ret);
    }

    pub async fn remove_from_wishlist(&self, user_id:&str, series:&str, card:&str) -> Option<mongodb::error::Error> {
        let collection = get_wishlist_collection(&self.db_client);
        
        let series_search = series_to_search(series);

        let res: Result<Option<Document>, _> = 
            collection.find_one_and_update( 
                doc!{"id": user_id, "series.search": series_search}, 
                doc!{"$pull": { "series.$[elem].cards": card }}, 
                FindOneAndUpdateOptions::builder()
                .array_filters(vec![doc! {"elem.name": series }])
                .build()
            ).await;

        match res {
            Ok(_) => None,
            Err(err) => Some(err)
        }
    }

    async fn user_has_series(&self, user_id:&str, series:&str) -> bool {
        let collection = get_wishlist_collection(&self.db_client);

        match collection.find_one(
            doc! {"id": user_id, "series.name": series},
            None
        ).await {
            Ok(x) => x.is_some(),
            Err(_) => false,
        }
    }
}


const WISHLIST_DATABASE_NAME : &str = "better_wishlist";
const WISHLIST_COLLECTION_NAME : &str = "wishlist";

fn get_wishlist_collection(client: &mongodb::Client) -> mongodb::Collection<Document> {
    let database = client.database(WISHLIST_DATABASE_NAME);
    let collection: mongodb::Collection<Document> = database.collection(WISHLIST_COLLECTION_NAME);

    return collection;
}