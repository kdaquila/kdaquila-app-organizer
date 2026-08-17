//! `const` and `type` are ungoverned, so this file groups by topic.

pub type Seconds = u64;

pub const TOKEN_TTL: Seconds = 900;
pub const REFRESH_TTL: Seconds = 86_400;
pub const MAX_ATTEMPTS: u8 = 5;
