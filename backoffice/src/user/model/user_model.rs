use crate::user::role::Role;

#[derive(Debug)]
pub struct UserIdContext {
    #[allow(dead_code)]
    pub id: i64,
    pub username: String,
    pub role: Role,
}

pub struct IdPassword {
    pub id: i64,
    pub password: Box<[u8]>,
}
