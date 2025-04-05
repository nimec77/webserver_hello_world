use std::sync::Arc;

use axum::{Router, extract::State, routing::get};
use tokio::net::TcpListener;

use crate::{app::query::get_hello_world::Repository, di::Container};

pub struct Server<R>
where
    R: Repository + Send + Sync + 'static,
{
    port: u16,
    container: Arc<Container<R>>,
}

impl<R> Server<R>
where
    R: Repository + Send + Sync + 'static,
{
    pub fn new(port: u16, container: Arc<Container<R>>) -> Self {
        Self { port, container }
    }

    pub async fn run(self) {
        let app = get_router(self.container);
        let listener = TcpListener::bind(format!("0.0.0.0:{}", self.port))
            .await
            .unwrap();

        axum::serve(listener, app).await.unwrap();
    }
}

async fn handler<R>(State(container): State<Arc<Container<R>>>) -> &'static str
where
    R: Repository + Send + Sync + 'static,
{
    container.hello_world_query.execute().await
}

pub fn get_router<R>(container: Arc<Container<R>>) -> Router
where
    R: Repository + Send + Sync + 'static,
{
    Router::new()
        .route("/hello", get(handler))
        .with_state(container)
}

#[cfg(test)]
mod tests {
    use crate::app::query::get_hello_world::InMemoryRepository;

    use super::*;
    use axum::{body::Body, extract::Request, http::StatusCode};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    fn setup() -> Arc<Container<InMemoryRepository>> {
        let repository = InMemoryRepository;
        
        Arc::new(Container::new(repository))
    }

    #[tokio::test]
    async fn test_get_router() {
        // Given
        let container = setup();
        let app = get_router(container);

        // When
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/hello")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Then
        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        assert_eq!(body_str, "Hello, world!");
    }

    #[tokio::test]
    async fn not_found() {
        // Given
        let container = setup();
        let app = get_router(container);

        // When
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/not-found")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Then
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
