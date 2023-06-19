use api::{router, Configuration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber)?;

    let (route, bind) = (router(), Configuration::address());
    let app = axum::Server::bind(&bind)
        .serve(route.into_make_service())
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c()
                .await
                .expect("expect tokio signal ctrl-c");
            tracing::info!("stopping app...");
        });

    tracing::info!("start app in {}", bind);
    if let Err(err) = app.await {
        tracing::error!("server error: {}", err);
        Err(err.into())
    } else {
        Ok(())
    }
}
