pub struct Session {
    pub user: String,
}

impl Session {
    pub fn new(user: &str) -> Session {
        Session {
            user: user.to_string(),
        }
    }
}
