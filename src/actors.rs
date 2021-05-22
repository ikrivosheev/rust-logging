use actix::prelude::*;


#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct NewRecord {
    level: String,
    time: u64,
    message: String
}


impl NewRecord {
    pub fn new(level: &str, time: u64, message: &str) -> Self {
        NewRecord {
            level: level.to_string(),
            time: time,
            message: message.to_string()
        }
    }
}

pub struct Inserter;

impl Inserter {
    pub fn new() -> Self {
        Inserter {}
    }

}

impl Actor for Inserter {
    type Context = Context<Self>;
}

impl Handler<NewRecord> for Inserter {
    type Result = ();

    fn handle(&mut self, msg: NewRecord, _: &mut Self::Context) -> Self::Result {
        println!("{:?}", msg);
    }
}
