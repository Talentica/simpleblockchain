pub trait Message {
    const TOPIC: &'static str;
    fn handler(&self);
    fn topic(&self) -> String {
        String::from(Self::TOPIC)
    }
}

#[derive(Debug)]
pub struct TransactionCreate {}

impl Message for TransactionCreate {
    const TOPIC: &'static str = "txn-create";
    fn handler(&self) {
        println!("i am txn create");
    }
}

#[derive(Debug)]
pub struct BlockCreate {}

impl Message for BlockCreate {
    const TOPIC: &'static str = "block-create";
    fn handler(&self) {
        println!("i am blockcreate");
    }
}

#[derive(Debug)]
pub enum MessageTypes {
    TransactionCreate(TransactionCreate),
    BlockCreate(BlockCreate),
    BlockFinalize,
    MsgTest,
}
