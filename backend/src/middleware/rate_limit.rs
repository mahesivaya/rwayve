use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse,
};
use futures::future::{ok, Ready, LocalBoxFuture};
use std::{task::{Context, Poll}, rc::Rc, collections::HashMap, time::{Instant, Duration}};
use std::sync::Mutex;

pub struct RateLimitMiddleware;

static RATE_LIMIT: once_cell::sync::Lazy<Mutex<HashMap<String, (u32, Instant)>>> =
    once_cell::sync::Lazy::new(|| Mutex::new(HashMap::new()));

const MAX_REQUESTS: u32 = 100;
const WINDOW: Duration = Duration::from_secs(60);

impl<S, B> Transform<S, ServiceRequest> for RateLimitMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = RateLimitService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(RateLimitService { service: Rc::new(service) })
    }
}

pub struct RateLimitService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for RateLimitService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let key = req
            .extensions()
            .get::<i32>() // user_id from AuthMiddleware
            .map(|id| format!("user:{}", id))
            .unwrap_or_else(|| {
                req.connection_info()
                    .realip_remote_addr()
                    .unwrap_or("unknown")
                    .to_string()
            });

        let mut map = RATE_LIMIT.lock().unwrap();
        let entry = map.entry(key.clone()).or_insert((0, Instant::now()));

        if entry.1.elapsed() > WINDOW {
            *entry = (0, Instant::now());
        }

        entry.0 += 1;

        if entry.0 > MAX_REQUESTS {
            return Box::pin(async {
                Ok(req.into_response(
                    HttpResponse::TooManyRequests().body("Rate limit exceeded"),
                ))
            });
        }

        let srv = self.service.clone();
        Box::pin(async move { srv.call(req).await })
    }
}