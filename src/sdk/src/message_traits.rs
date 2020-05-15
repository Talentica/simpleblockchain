pub trait Message {
    const TOPIC: &'static str;
    const MODULE_TOPIC: &'static str;

    fn handler(&self);

    fn topic(&self) -> String {
        String::from(Self::TOPIC)
    }

    fn module_topic(&self) -> String {
        String::from(Self::MODULE_TOPIC)
    }

    fn get_topics(&self) -> Vec<String> {
        let mut ret: Vec<String> = Vec::new();
        ret.push(self.module_topic());
        ret.push(self.topic());
        ret
    }
}
