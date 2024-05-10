use std::{sync::Arc, vec};

use mongodb::{self, bson::{doc, Bson, Document}, error::Error, options::{ClientOptions, UpdateOptions}, Client};
use serenity::futures::TryStreamExt;

use crate::component::logger::Logger;

pub struct WishlistDB<T> 
    where T: Logger 
{
    db_client: mongodb::Client,
    logger: Arc<T>
}

pub async fn init_db<T>(logger: Arc<T>, uri: impl AsRef<str>) -> Result<WishlistDB<T>, Error> 
    where T: Logger 
{
    // Create a new client and connect to the server
    let mut client_options = ClientOptions::parse_async(uri).await?;
    client_options.max_connecting = Some(3);

    // let client = mongodb::Client::with_uri_str(uri).await;
    let client = Client::with_options(client_options);

    return client.map(|db_client| WishlistDB{db_client, logger});
}

impl <T> WishlistDB<T> 
    where T: Logger 
{
    pub async fn get_wishlisted_users(&self, series:&str, card_name:&str) -> Result<Vec<String>, Error> {
        let collection = get_wishlist_collection(&self.db_client);

        let series_search = series_to_search_term(series);
        let card_search = card_to_search_term(card_name);

        let res =
            collection.find(
                doc!{ "series": { "$elemMatch": {"search": series_search, "cards.search": card_search}}},
                // Some(FindOptions::builder().projection(doc! {"id": 1}).build() )
                None
            ).await;

        if let Err(err) = res {
            self.logger.log_error(err.to_string());
            return Err(err);
        }

        let cursor = res.unwrap();

        let ret = 
            cursor.try_collect().await
                  .unwrap_or_else(|_| vec![])
                  .iter()
                  .map(|doc| doc.get_str("id").unwrap_or("").to_string())
                  .collect();

        return Ok(ret);
    }

    pub async fn add_all_to_wishlist(&self, user_id:&str, series:&str, mut card_names:Vec<&str>) -> Result<i32, Error> {
        let collection = get_wishlist_collection(&self.db_client);
        
        let series_search = series_to_search_term(series);

        let initial_amount;
        if !self.user_has_series(user_id, series).await {
            let res = collection.update_one(
                doc! {"id": user_id},
                doc! {"$addToSet": { "series": { "name": series, "search": &series_search, "cards": [] }}},
                UpdateOptions::builder().upsert(true).build()
            ).await;
            
            if res.is_err() {
                return Err(res.unwrap_err());
            }

            initial_amount = 0;
        } else {
            initial_amount = self.get_user_wishlisted_cards_count(user_id, series).await;
        };

        // avoid processing duplicate cards
        card_names.sort();
        card_names.dedup();

        // create card objects to insert
        let cards : Vec<Document> = card_names.iter()
            .map(|card| doc!{"name": card, "search": card_to_search_term(card)})
            .collect();

        // add all cards in one go
        let res = collection.update_one( 
            doc!{"id": user_id, "series.search": &series_search}, 
            doc!{"$addToSet": { "series.$[elem].cards": doc!{"$each": cards} }}, 
            UpdateOptions::builder()
            .upsert(true)
            .array_filters(vec![doc! {"elem.search": series_search }])
            .build()
        ).await;

        match res {
            Ok(_) => {
                let curr_amount = self.get_user_wishlisted_cards_count(user_id, series).await;
                Ok(curr_amount - initial_amount)
            },
            Err(err) => {
                self.logger.log_error(err.to_string());
                Err(err)
            }
        }
    }

    pub async fn remove_all_from_wishlist(&self, user_id:&str, series:&str, card_names:Vec<&str>) -> Result<i32, Error> {
        let collection = get_wishlist_collection(&self.db_client);
        
        let initial_amount = self.get_user_wishlisted_cards_count(user_id, series).await;
        let series_search = series_to_search_term(series);
        let cards_search : Vec<String> = card_names.iter()
            .map(|card| card_to_search_term(card))
            .collect();

        let res = 
            collection.update_one( 
                doc!{"id": user_id, "series.search": &series_search}, 
                doc!{"$pull": { "series.$[elem].cards": doc!{"search": {"$in": cards_search}} }}, 
                UpdateOptions::builder()
                .array_filters(vec![doc! {"elem.search": series_search }])
                .build()
            ).await;

        match res {
            Ok(_) => {
                let curr_amount = self.get_user_wishlisted_cards_count(user_id, series).await;

                if curr_amount == 0 {
                    self.remove_series_from_wishlist(user_id, series).await;
                }

                Ok(initial_amount - curr_amount)
            },
            Err(err) => Err(err)
        }
    }

    // pub async fn get_user_wishlist(&self, user_id: &str) -> Vec<(String, Vec<String>)> {
    //     let collection = get_wishlist_collection(&self.db_client);
        
    //     let Ok(user_opt) =
    //         collection.find_one(
    //             doc!{"id": user_id},
    //             None
    //         ).await 
    //         else { todo!() };

    //     if user_opt.is_none() {
    //         vec![]
    //     } else {
    //         let series_doc_opt = user_opt.unwrap();
    //         let Ok(series) = series_doc_opt
    //             .get_array("series")
    //             else { todo!() };

    //         series.iter()
    //             .map(Bson::as_document)
    //             .map(Option::unwrap)
    //             .map(flatten_series_document)
    //             .filter(Option::is_some)
    //             .map(Option::unwrap)
    //             .collect()
    //     }
    // }

    // pub async fn get_user_wishlisted_series_count(&self, user_id: &str) -> i32 {
    //     let collection = get_wishlist_collection(&self.db_client);

    //     let Ok(cursor) =
    //         collection.aggregate(
    //             [
    //                 doc!{"$match": { "id": user_id }},
    //                 doc!{"$project": { "count": {"$size": "$series.name"}}}
    //             ],
    //             None
    //         ).await 
    //         else { todo!() };

    //     let Ok(count) = cursor.current().get_i32("count")
    //     else{
    //         // TODO: log error
    //         return 0;
    //     };

    //     count
    // }

    // pub async fn get_user_wishlisted_series(&self, user_id: &str, start: i32, end: i32) -> Vec<String> {
    //     let collection = get_wishlist_collection(&self.db_client);

    //     let Ok(cursor) =
    //         collection.aggregate(
    //             [
    //                 doc!{"$match": { "id": "234822770385485824" }},
    //                 doc!{"$project": { "series": { "$slice": ["$series.name", start, end]}}}
    //             ],
    //             None
    //         ).await 
    //         else { todo!() };

    //     // cursor.advance();
    //     // let Ok(x) = cursor.current().get_array("series")
    //     // else {
    //     //     // TODO log error
    //     //     return vec![];
    //     // };
        
    //     todo!()
    // }

    pub async fn get_user_wishlisted_series(&self, user_id: &str/*, start: i32, end: i32*/) -> Vec<String> {
        let collection = get_wishlist_collection(&self.db_client);

        let Ok(mut cursor) =
            collection.aggregate(
                [
                    doc!{"$match": { "id": user_id }},
                    doc!{"$project": { "series": "$series.name"}}
                    // doc!{"$project": { "series": { "$slice": ["$series.name", start, end]}}}
                ],
                None
            ).await 
            else { todo!() };

        match cursor.advance().await {
            Ok(true) => (),
            Ok(false) => return vec![], // TODO maybe log a warning?
            Err(err) => {
                self.logger.log_error(format!("get_user_wishlisted_series: {}", err.to_string()));
                return vec![];
            },
        }

        let Ok(x) = cursor.deserialize_current()
        else {
            // TODO log error
            return vec![];
        };

        let series_vec = x.get_array("series");
        if let Err(_) = series_vec {
            // TODO log error
            return vec![];
        }

        let mut ret = vec![];
        for opt_series in series_vec.unwrap().iter().map(Bson::as_str) {
            match opt_series {
                Some(series) => ret.push(series.to_string()),
                None => {
                    self.logger.log_error("get_user_wishlisted_series: could not parse Bson as string")
                }
            }
        }

        return ret;
    }

    pub async fn get_user_wishlisted_cards_count(&self, user_id: &str, series: &str) -> i32 {
        let collection = get_wishlist_collection(&self.db_client);

        let series_search = series_to_search_term(series);

        let res =
            collection.aggregate(
                [
                    doc!{ "$match": { "id": user_id, "series.search": &series_search }},
                    doc!{ "$project": { "series":
                    { "$filter": 
                        { "input":"$series",
                          "as": "serie",
                          "cond": 
                            { "$eq": ["$$serie.search", series_search] }
                        }
                    }}},
                    doc! { "$project": {
                        "count": { "$size": { "$arrayElemAt": ["$series.cards", 0] } }
                      }}
                ],
                None
            ).await;

        
        if let Err(err) = res { 
            self.logger.log_error(format!("get_user_wishlisted_cards_count: {}", err.to_string()));
            return 0; 
        };

        let mut cursor = res.unwrap(); 
        if cursor.advance().await.unwrap() {
            match cursor.current().get_i32("count") {
                Ok(count) => count,
                Err(err) => {
                    self.logger.log_error(format!("get_user_wishlisted_cards_count: {}", err.to_string()));
                    0
                }
            }
        } else {
            0
        }
    }

    pub async fn get_user_wishlisted_cards(&self, user_id: &str, series: &str) -> Vec<String> {
        let collection = get_wishlist_collection(&self.db_client);

        let series_search = series_to_search_term(series);

        let res =
            collection.aggregate(
                [
                    doc!{ "$match": { "id": user_id, "series.search": &series_search }},
                    doc!{ "$project": { "series":
                    { "$filter": 
                        { "input":"$series",
                          "as": "serie",
                          "cond": 
                            { "$eq": ["$$serie.search", series_search] }
                        }
                    }}},
                    doc! { "$project": {
                        "cards": { "$map": { "input": { "$arrayElemAt": ["$series.cards", 0]}, "as": "card", "in": "$$card.name" } }
                      }}
                ],
                None
            ).await;

        
        if let Err(err) = res { 
            self.logger.log_error(format!("get_user_wishlisted_cards_count: {}", err.to_string()));
            return vec![]; 
        };

        let mut cursor = res.unwrap(); 
        if cursor.advance().await.unwrap() {
            match cursor.current().get_array("cards") {
                Ok(cards) => {
                    
                    let mut vec_string: Vec<String> = Vec::new();
                    for x in cards.into_iter() {
                        match x {
                            Err(_) => (),
                            Ok(card) => {
                                vec_string.push(card.as_str().unwrap().to_string());
                            }
                        }
                    }
                    
                    vec_string
                },
                Err(err) => {
                    self.logger.log_error(format!("get_user_wishlisted_cards_count: {}", err.to_string()));
                    vec![]
                }
            }
        } else {
            vec![]
        }
    }

    async fn user_has_series(&self, user_id: &str, series: &str) -> bool {
        let collection = get_wishlist_collection(&self.db_client);

        let series_search = series_to_search_term(series);

        match collection.find_one(
            doc! {"id": user_id, "series.search": series_search},
            None
        ).await {
            Ok(x) => x.is_some(),
            Err(_) => false,
        }
    }

    async fn remove_series_from_wishlist(&self, user_id:&str, series:&str) {
        let collection = get_wishlist_collection(&self.db_client);

        let series_search = series_to_search_term(series);

        let res = 
            collection.update_one( 
                doc!{"id": user_id, "series.search": &series_search}, 
                doc!{"$pull": { "series": {"search": series_search}}}, 
                None
            ).await;

        if let Err(err) = res {
            self.logger.log_error(format!("remove_series_from_wishlist: {}", err.to_string()))
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

pub fn series_to_search_term(name: &str) -> String {
    let mut search = name.to_lowercase();
    search.truncate(32);
    search
}

pub fn card_to_search_term(name: &str) -> String {
    let mut search = name.to_lowercase();
    search.truncate(16);
    search
}