use futures::{channel::mpsc::*, executor::*, future, prelude::*, task::*};
use p2plib::constants::*;
use p2plib::messages::*;
use std::error::Error;

use std::sync::Arc;
use std::sync::Mutex;

#[derive(Debug)]
pub struct NodeMsgProcessor {
    _tx: Sender<Option<NodeMessageTypes>>,
    pub _rx: Receiver<Option<NodeMessageTypes>>,
}

impl NodeMsgProcessor {
    pub fn new() -> Self {
        let (mut tx, mut rx) = channel::<Option<NodeMessageTypes>>(1024);
        NodeMsgProcessor { _tx: tx, _rx: rx }
    }
    pub fn start(&mut self) {
        //, rx: &'static mut Receiver<Option<NodeMessageTypes>>) {
        // let thread_handle = thread::spawn(move || {
        block_on(future::poll_fn(move |cx: &mut Context| {
            loop {
                match self._rx.poll_next_unpin(cx) {
                    Poll::Ready(Some(msg)) => {
                        println!("msg received {:?}", msg);
                        match msg {
                            None => println!("Empty msg received !"),
                            Some(msgtype) => {
                                match msgtype {
                                    NodeMessageTypes::BlockCreate(data) => {
                                        println!(
                                            "Blockcreate msg in NodeMsgProcessor with data {:?}",
                                            data
                                        );
                                        //TODO
                                        //Write msg processing code
                                    }
                                    NodeMessageTypes::TransactionCreate(data) => {
                                        println!("TransactionCreate msg in NodeMsgProcessor with data {:?}", data);
                                        //TODO
                                        //Write msg processing code
                                    }
                                }
                            }
                        }
                    }
                    Poll::Ready(None) => {
                        println!("channel closed !");
                        return Poll::Ready(1);
                    }
                    Poll::Pending => break,
                }
            }
            Poll::Pending
        }));
    }
}

lazy_static! {
    pub static ref thread_safe_nodemsgprocessor: Arc<Mutex<NodeMsgProcessor>> =
        Arc::new(Mutex::new(NodeMsgProcessor::new()));
}
