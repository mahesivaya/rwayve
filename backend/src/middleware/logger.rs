use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures::future::{ok, Ready, LocalBoxFuture};
use std::task::{Context, Poll};
use std::rc::Rc;
use std::time::Instant;
use log::{info, error};

pub struct LoggerMiddleware;

impl<S, B> Transform<S, ServiceRequest> for LoggerMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Transform = LoggerMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(LoggerMiddlewareService {
            service: Rc::new(service),
        })
    }
}

pub struct LoggerMiddlewareService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for LoggerMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let start = Instant::now();

        let method = req.method().to_string();
        let path = req.path().to_string();

        // 🔥 Extract user_id from JWT if available
        let user_id = req
            .headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
            .and_then(|token| crate::security::jwt::decode_jwt(token))
            .map(|claims| claims.sub);

        let srv = self.service.clone();

        Box::pin(async move {
            let res = srv.call(req).await;

            let duration = start.elapsed().as_millis();

            match &res {
                Ok(response) => {
                    info!(
                        "📡 {} {} | status={} | user_id={:?} | {}ms",
                        method,
                        path,
                        response.status(),
                        user_id,
                        duration
                    );
                }
                Err(err) => {
                    error!(
                        "❌ {} {} | error={} | user_id={:?} | {}ms",
                        method,
                        path,
                        err,
                        user_id,
                        duration
                    );
                }
            }

            res
        })
    }
}