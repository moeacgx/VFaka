use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::{web, Error, HttpMessage};
use std::future::{ready, Future, Ready};
use std::pin::Pin;
use std::rc::Rc;

use aff_common::config::AppConfig;
use aff_common::error::AppError;
use aff_core::services::admin_service;

pub struct JwtAuth;

impl<S, B> Transform<S, ServiceRequest> for JwtAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = JwtAuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(JwtAuthMiddleware {
            service: Rc::new(service),
            require_super_admin: false,
        }))
    }
}

pub struct SuperAdminAuth;

impl<S, B> Transform<S, ServiceRequest> for SuperAdminAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = JwtAuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(JwtAuthMiddleware {
            service: Rc::new(service),
            require_super_admin: true,
        }))
    }
}

pub struct JwtAuthMiddleware<S> {
    service: Rc<S>,
    require_super_admin: bool,
}

impl<S, B> Service<ServiceRequest> for JwtAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);
        let require_super_admin = self.require_super_admin;

        Box::pin(async move {
            let config = req
                .app_data::<web::Data<AppConfig>>()
                .ok_or_else(|| AppError::Internal("AppConfig not found".to_string()))?;

            let token = extract_bearer_token(&req)
                .ok_or_else(|| AppError::Unauthorized("Missing authorization token".to_string()))?;

            let claims = admin_service::verify_token(&token, &config.jwt.secret)?;

            if require_super_admin && claims.role != "super_admin" {
                return Err(AppError::Forbidden("Super admin access required".to_string()).into());
            }

            req.extensions_mut().insert(claims);

            service.call(req).await
        })
    }
}

fn extract_bearer_token(req: &ServiceRequest) -> Option<String> {
    let header = req.headers().get("Authorization")?.to_str().ok()?;
    header
        .strip_prefix("Bearer ")
        .map(|token| token.to_string())
}
