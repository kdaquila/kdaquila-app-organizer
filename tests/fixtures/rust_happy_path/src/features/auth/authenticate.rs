use super::session::Session;

pub fn authenticate(user: &str) -> Option<Session> {
    check(user).then(|| Session::new(user))
}

fn check(user: &str) -> bool {
    !user.is_empty()
}
