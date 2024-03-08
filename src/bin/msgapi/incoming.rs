pub struct msg {}

impl msg {
    pub fn new() -> msg {
        msg {}
    }
    pub fn decode(&self, msg: &str) -> String {
        //TODO: Implement this
    }

    pub fn is_event(&self, msg: &str) -> bool {
        //TODO: Implement this
        false
    }

    pub fn check_signature(&self, msg: &str) -> bool {
        //TODO: Implement this
        false
    }

    pub fn exists(&self, msg: &str) -> bool {
        //TODO: Implement this
        false
    }
}
