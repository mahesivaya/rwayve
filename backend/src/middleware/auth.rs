use actix_web::body::EitherBody;
use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse,
};
use futures::future::{ok, Ready, LocalBoxFuture};
use std::task::{Context, Poll};
use std::rc::Rc;

pub struct AuthMiddleware;

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Transform = AuthMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthMiddlewareService {
            service: Rc::new(service),
        })
    }
}

pub struct AuthMiddlewareService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
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

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let auth_header = req.headers().get("Authorization");

        let token = match auth_header
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
        {
            Some(t) => t,
            None => {
                return Box::pin(async {
                    Ok(req.into_response(
                        HttpResponse::Unauthorized().body("Missing token"),
                    ))
                });
            }
        };

        // 🔥 Decode JWT (your existing logic)
        let claims = match crate::security::jwt::decode_jwt(token) {
            Some(c) => c,
            None => {
                return Box::pin(async {
                    Ok(req.into_response(
                        HttpResponse::Unauthorized().body("Invalid token"),
                    ))
                });
            }
        };

        // 🔥 Store user_id in request extensions
        req.extensions_mut().insert(claims.sub);

        let srv = self.service.clone();

        Box::pin(async move {
            let res = srv.call(req).await;
            res
        })
    }
}