use std::num::NonZeroU32;

use governor::{Quota, RateLimiter};

use super::LoginRateLimiter;

/// 5 login attempts per IP per minute.
pub fn login_rate_limiter() -> LoginRateLimiter {
    let quota =
        Quota::per_minute(NonZeroU32::new(5).expect("non-zero quota for login rate limiter"));
    RateLimiter::keyed(quota)
}
