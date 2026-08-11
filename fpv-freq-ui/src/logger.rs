pub fn setup_logger() {
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    let registry = tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(
                    if cfg!(debug_assertions) {
                        tracing::Level::DEBUG
                    } else {
                        tracing::Level::INFO
                    }
                    .into(),
                )
                .from_env()
                .expect("default level is set")
                .add_directive("hyper_util=warn".parse().unwrap())
                .add_directive("winit=info".parse().unwrap())
                .add_directive("sctk=info".parse().unwrap())
                .add_directive("naga=info".parse().unwrap())
                .add_directive("wgpu_hal=info".parse().unwrap())
                .add_directive("wgpu_core=info".parse().unwrap())
                .add_directive("reqwest=warn".parse().unwrap()),
        );

    registry.init();
}
