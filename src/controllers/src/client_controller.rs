extern crate services;
use actix_rt::System;
use actix_web::{dev::Server, get, middleware, post, web, App, HttpServer, Responder};
use futures::channel::mpsc::*;
use p2plib::messages::MessageTypes;
use services::client_services::ClientServices;
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::Mutex;

pub trait Controller {
    //fn new() -> Self;
    fn start(&mut self, sender: Sender<Option<MessageTypes>>) -> bool;
    fn stop(&self);
}

// #[get("/client/submit_transaction")]
#[post("/client/submit_transaction")]
async fn submit_transaction_controller(
    transaction: web::Bytes,
    app_state: web::Data<Mutex<AppState>>,
) -> impl Responder {
    // call client service handler
    ClientServices::submit_transaction_service(
        transaction,
        &mut app_state.lock().unwrap().txn_sender,
    )
    // HttpResponse::Ok().body("txn added in the pool")
}

#[get("/client/fetch_pending_transaction")]
async fn fetch_pending_transaction_controller(transaction_hash: web::Bytes) -> impl Responder {
    ClientServices::fetch_pending_transaction_service(transaction_hash)
}

#[get("/client/fetch_confirm_transaction")]
async fn fetch_confirm_transaction_controller(transaction_hash: web::Bytes) -> impl Responder {
    ClientServices::fetch_confirm_transaction_service(transaction_hash)
}

#[get("/client/fetch_state")]
async fn fetch_state_controller(address: web::Bytes) -> impl Responder {
    ClientServices::fetch_state_service(address)
}

#[get("/peer/fetch_block")]
async fn fetch_block_peer_controller(address: web::Bytes) -> impl Responder {
    ClientServices::fetch_block_peer_service(address)
}

#[get("/peer/fetch_latest_block")]
async fn fetch_latest_block_peer_controller() -> impl Responder {
    ClientServices::fetch_latest_block_peer_service()
}

#[get("/client/fetch_block")]
async fn fetch_block_controller(address: web::Bytes) -> impl Responder {
    ClientServices::fetch_block_service(address)
}

#[get("/client/fetch_latest_block")]
async fn fetch_latest_block_controller() -> impl Responder {
    ClientServices::fetch_latest_block_service()
}

#[get("/peer/fetch_blockchain_length")]
async fn fetch_blockchain_length_peer_controller() -> impl Responder {
    ClientServices::fetch_blockchain_length_peer_service()
}

#[get("/client/fetch_blockchain_length")]
async fn fetch_blockchain_length_controller() -> impl Responder {
    ClientServices::fetch_blockchain_length_service()
}

//May be useful for storing forwarding/db_layer related info
#[allow(dead_code)]
struct AppState {
    txn_sender: Sender<Option<MessageTypes>>,
}

// pub static mut APP_STATE: AppState = AppState{ txn_sender: None};

pub struct ClientController {
    srvr_addr: SocketAddr,
    srvr: Option<Server>,
}

impl ClientController {
    pub fn new(host: &String, port: u32) -> Self {
        //to enable logging use below setting
        std::env::set_var("RUST_LOG", "actix_web=info,actix_server=trace");
        env_logger::init();
        let addr = format!("{}:{}", host, port)
            .to_socket_addrs()
            .expect("Unable to resolve the address")
            .next()
            .unwrap();
        ClientController {
            srvr_addr: addr,
            srvr: None,
        }
    }
}

impl Controller for ClientController {
    fn start(&mut self, sender: Sender<Option<MessageTypes>>) -> bool {
        let sys = System::new("TransactionService");
        println!("Starting api_service at {:?}", self.srvr_addr);
        let app_data = web::Data::new(Mutex::new(AppState { txn_sender: sender }));
        self.srvr = Some(
            HttpServer::new(move || {
                App::new()
                    .app_data(app_data.clone())
                    // enable logger
                    .wrap(middleware::Logger::default())
                    //.service(web::resource("/index.html").to(|| async { "Hello world!" }))
                    .service(submit_transaction_controller)
                    .service(fetch_confirm_transaction_controller)
                    .service(fetch_pending_transaction_controller)
                    .service(fetch_state_controller)
                    .service(fetch_block_controller)
                    .service(fetch_latest_block_controller)
                    .service(fetch_block_peer_controller)
                    .service(fetch_latest_block_peer_controller)
                    .service(fetch_blockchain_length_peer_controller)
                    .service(fetch_blockchain_length_controller)
            })
            .bind(self.srvr_addr)
            .unwrap()
            .shutdown_timeout(5)
            .run(),
        );

        let _ = sys.run();
        true
    }

    fn stop(&self) {
        System::new("").block_on(self.srvr.as_ref().unwrap().stop(true));
    }
}
