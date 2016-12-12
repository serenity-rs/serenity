use std::collections::HashMap;

#[doc(hidden)]
pub struct Ratelimit {
    pub delay: i64,
    pub limit: Option<(i64, i32)>
}

#[doc(hidden)]
pub struct MemberRatelimit {
    pub count: i32,
    pub last_time: i64,
    pub set_time: i64
}

#[doc(hidden)]
pub struct Bucket {
    pub ratelimit: Ratelimit,
    pub limits: HashMap<u64, MemberRatelimit>
}
