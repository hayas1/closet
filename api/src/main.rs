use api::{configuration::Configuration, router, with_auth};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber)?;

    let configuration = Configuration::new(Default::default());
    let (router, bind) = (router(configuration.base_url()), configuration.address());
    let app = axum::Server::bind(&bind)
        .serve(with_auth(router, configuration.clone()).await?.into_make_service())
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c().await.expect("expect tokio signal ctrl-c");
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
