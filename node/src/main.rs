use actix_web::{web, App, HttpResponse, HttpServer, Responder, post};
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use std::sync::{Arc, Mutex};
use hex::encode;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Block {
    index: u64,
    previous_hash: String,
    timestamp: u128,
    data: String,
    hash: String,
}

impl Block {
    fn new(index: u64, previous_hash: String, timestamp: u128, data: String, hash: String) -> Self {
        Block {
            index,
            previous_hash,
            timestamp,
            data,
            hash,
        }
    }

    fn hash(&self) -> String {
        let mut hasher = Sha256::new();
        let input = format!("{}{}{}{}", self.index, self.previous_hash, self.timestamp, self.data);
        hasher.update(input.as_bytes());
        encode(hasher.finalize())
    }
}

type Blockchain = Arc<Mutex<Vec<Block>>>;

#[post("/mine")]
async fn mine(data: web::Json<String>, blockchain: web::Data<Blockchain>) -> impl Responder {
    let mut bc = blockchain.lock().unwrap();
    let last_block = bc.last().unwrap().clone();

    let new_block = Block::new(
        last_block.index + 1,
        last_block.hash.clone(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
        data.into_inner(),
        String::new(),
    );

    let hash = new_block.hash();
    let mined_block = Block::new(new_block.index, new_block.previous_hash, new_block.timestamp, new_block.data, hash);

    bc.push(mined_block.clone());
    HttpResponse::Ok().json(mined_block)
}

#[post("/chain")]
async fn get_chain(blockchain: web::Data<Blockchain>) -> impl Responder {
    let chain = blockchain.lock().unwrap();
    HttpResponse::Ok().json(&*chain)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let genesis_block = Block::new(
        0,
        String::from("0"),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
        String::from("Genesis block"),
        String::from("0"),
    );

    let blockchain = Arc::new(Mutex::new(vec![genesis_block]));

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(blockchain.clone()))
            .service(mine)
            .service(get_chain)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}