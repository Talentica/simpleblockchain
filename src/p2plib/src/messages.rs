pub trait Message {
    const TOPIC: &'static str;
    fn handler(&self);
}

pub struct TransactionCreate {}

impl Message for TransactionCreate {
    const TOPIC: &'static str = "txn-create";
    fn handler(&self) {}
}

pub struct BlockCreate {}
impl Message for BlockCreate {
    const TOPIC: &'static str = "block-create";
    fn handler(&self) {}
}
pub enum MessageTypes {
    TransactionCreate(TransactionCreate),
    BlockCreate(BlockCreate),
    BlockFinalize,
    MsgTest,
}
