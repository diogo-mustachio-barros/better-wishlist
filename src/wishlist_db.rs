use std::vec;

use mongodb::{self, bson::{doc, Bson, Document}, error::Error, options::{ClientOptions, FindOneAndUpdateOptions}, Client};
use serenity::futures::TryStreamExt;

pub struct WishlistDB {
    db_client: mongodb::Client
}

pub async fn init_db(uri: impl AsRef<str>) -> Result<WishlistDB, Error> {
    // Create a new client and connect to the server
    let mut client_options = ClientOptions::parse_async(uri).await?;
    client_options.max_connecting = Some(3);

    // let client = mongodb::Client::with_uri_str(uri).await;
    let client = Client::with_options(client_options);

    return client.map(|db_client| WishlistDB{db_client});
}

impl WishlistDB {
    pub async fn add_all_to_wishlist(&self, user_id:&str, series:&str, card_names:Vec<&str>) -> Option<Error> {
        let collection = get_wishlist_collection(&self.db_client);
        
        let series_search = to_search_term(series);
        
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

        let cards : Vec<Document> = card_names.iter()
            .map(|card| doc!{"name": card, "search": to_search_term(card)})
            .collect();

        let res = collection.find_one_and_update( 
            doc!{"id": user_id, "series.search": &series_search}, 
            doc!{"$addToSet": { "series.$[elem].cards": doc!{"$each": cards} }}, 
            FindOneAndUpdateOptions::builder()
            .upsert(true)
            .array_filters(vec![doc! {"elem.search": series_search }])
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
        let series_search = to_search_term(series);
        let card_search = to_search_term(card_name);

        let res =
            collection.find(
                doc!{"series.search": series_search, "series.cards.search": card_search},
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

    pub async fn remove_all_from_wishlist(&self, user_id:&str, series:&str, card_names:Vec<&str>) -> Option<mongodb::error::Error> {
        let collection = get_wishlist_collection(&self.db_client);
        
        let series_search = to_search_term(series);
        let cards_search : Vec<String> = card_names.iter()
            .map(|card| to_search_term(card))
            .collect();

        let res: Result<Option<Document>, _> = 
            collection.find_one_and_update( 
                doc!{"id": user_id, "series.search": &series_search}, 
                doc!{"$pullAll": { "series.$[elem].cards.search": cards_search }}, 
                FindOneAndUpdateOptions::builder()
                .array_filters(vec![doc! {"elem.name": series_search }])
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

    pub async fn get_user_wishlist(&self, user_id:&str) -> Vec<(String, Vec<String>)> {
        let collection = get_wishlist_collection(&self.db_client);
        
        let Ok(user_opt) =
            collection.find_one(
                doc!{"id": user_id},
                None
            ).await 
            else { todo!() };

        if user_opt.is_none() {
            vec![]
        } else {
            let series_doc_opt = user_opt.unwrap();
            let Ok(series) = series_doc_opt
                .get_array("series")
                else { todo!() };

            series.iter()
                .map(Bson::as_document)
                .map(Option::unwrap)
                .map(flatten_series_document)
                .filter(Option::is_some)
                .map(Option::unwrap)
                .collect()
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

fn flatten_series_document(series_doc:&Document) -> Option<(String, Vec<String>)> {
    let Ok(series_name) = series_doc.get_str("name")
        else { return None };

    let Ok(cards_bson) = series_doc.get_array("cards")
        else { return None };
    
    let cards:Vec<String> = cards_bson.iter()
        .map(Bson::as_document)
        .map(Option::unwrap)
        .map(flatten_card_document)
        .filter(Option::is_some)
        .map(Option::unwrap)
        .collect();

    Some((series_name.to_string(), cards))
}

fn flatten_card_document(card_doc:&Document) -> Option<String> {
    match card_doc.get_str("name") {
        Ok(card_name) => Some(card_name.to_owned()),
        Err(_) => None,
    }
}

pub fn to_search_term(name: &str) -> String {
    let mut search = name.to_lowercase();
    search.truncate(16);
    search
}