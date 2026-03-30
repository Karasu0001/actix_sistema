use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse, HttpMessage, body::BoxBody,
};
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Ready};

use crate::services::login_service::LoginService;

pub struct AuthMiddleware;

impl<S> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareService { service }))
    }
}

pub struct AuthMiddlewareService<S> {
    service: S,
}

impl<S> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let path = req.path().to_string();

        let rutas_publicas = vec!["/login", "/api/login", "/static"];
        let es_ruta_publica = rutas_publicas.iter().any(|ruta| path.starts_with(ruta));

        if es_ruta_publica {
            let fut = self.service.call(req);
            return Box::pin(async move {
                let res = fut.await?;
                Ok(res)
            });
        }

        if let Some(user) = LoginService::get_current_user_from_request(req.request()) {
            req.extensions_mut().insert(user.clone());
            
            let fut = self.service.call(req);
            return Box::pin(async move {
                let res = fut.await?;
                Ok(res)
            });
        }

        if path.starts_with("/api/") {
            Box::pin(async move {
                Ok(req.into_response(
                    HttpResponse::Unauthorized()
                        .json(serde_json::json!({
                            "success": false,
                            "msg": "Sesión expirada"
                        }))
                        .map_into_boxed_body()
                ))
            })
        } else {
            Box::pin(async move {
                Ok(req.into_response(
                    HttpResponse::Found()
                        .append_header(("Location", "/login"))
                        .finish()
                        .map_into_boxed_body()
                ))
            })
        }
    }
}