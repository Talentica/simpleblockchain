extern crate schema;

use actix_rt::System;
use actix_web::http::StatusCode;
use actix_web::{dev::Server, middleware, web, App, HttpResponse, HttpServer, Result};
use bytes::Bytes;
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::Mutex;

use prost::Message;
use schema::user_messages::*;
use std::io::Cursor;

use crate::Service;

async fn default_handler(body: Bytes) -> Result<HttpResponse> {
    //let app_name = &data.app_name;
    //let mut counter = data.counter.lock().unwrap();
    let body_clone = body.clone();
    let result = std::str::from_utf8(&body_clone);
    match result {
        Ok(body_str) => println!("Transaction: \n{:?}", body_str),
        Err(e) => println!("{:?}", e),
    }
    let txn = Transaction::decode(&mut Cursor::new(body.clone())).unwrap();
    println!("Requested Txn: \n{:?}", txn);

    Ok(HttpResponse::build(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        //.body(format!("Transaction Received successfully!!")))
        .body(Bytes::from(body))) //sample code to test successful message passing.
}

//May be useful for storing forwarding/db_layer related info
struct AppState {
    counter: Mutex<u32>,
}

pub struct TransactionService {
    srvr_addr: SocketAddr,
    srvr: Option<Server>,
}

impl TransactionService {
    pub fn new(host: &String, port: u32) -> Self {
        //to enable logging use below setting
        std::env::set_var("RUST_LOG", "actix_web=info,actix_server=trace");
        env_logger::init();
        let addr = format!("{}:{}", host, port)
            .to_socket_addrs()
            .expect("Unable to resolve the address")
            .next()
            .unwrap();

        TransactionService {
            srvr_addr: addr,
            srvr: None,
        }
    }
}

impl Service for TransactionService {
    fn start(&mut self) -> bool {
        let sys = System::new("TransactionService");
        println!("Starting api_service at {:?}", self.srvr_addr);
        self.srvr = Some(
            HttpServer::new(|| {
                App::new()
                    .data(AppState {
                        counter: Mutex::new(0),
                    })
                    // enable logger
                    .wrap(middleware::Logger::default())
                    //.service(web::resource("/index.html").to(|| async { "Hello world!" }))
                    .service(web::resource("/").to(default_handler))
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
