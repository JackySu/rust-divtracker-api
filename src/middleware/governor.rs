use rocket_governor::{Method, Quota, RocketGovernable, RocketGovernor};
use rocket::{request::{self, FromRequest}, State, outcome::Outcome, Request};

use std::ops::Deref;

pub struct RateLimitGuard;

impl<'r> RocketGovernable<'r> for RateLimitGuard {
    fn quota(_method: Method, _route_name: &str) -> Quota {
        Quota::per_minute(Self::nonzero(5u32))
    }
}

struct RocketGovernorGuard<'r>(&'r RocketGovernor<'r, RateLimitGuard>);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for RocketGovernorGuard<'r> {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let rate_limiter = request.guard::<&State<RocketGovernor<RateLimitGuard>>>().await.unwrap();
        let rate_limiter_guard = rate_limiter.inner().clone();
        Outcome::Success(RocketGovernorGuard(rate_limiter_guard))
    }
}

impl<'a> Deref for RocketGovernorGuard<'a> {
    type Target = RocketGovernor<'a, RateLimitGuard>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}