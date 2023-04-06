use honeybee::bar::Bar;
use honeybee::config::Config;
use honeybee::errors::*;
use honeybee::utils::{get_config_path, read_toml_config};

fn main() {
    let res = tokio::runtime::Builder::new_current_thread()
        .max_blocking_threads(5)
        .enable_all()
        .build()
        .unwrap()
        .block_on(async move {
            let config_file = get_config_path("~/Code/Repositories/honeybee/examples/config.toml")
                .error("Configuration file not found")?;
            let mut config: Config = read_toml_config(config_file)?;
            let modules = std::mem::take(&mut config.modules);
            let mut bar = Bar::new(config.general);

            for module in modules {
                bar.add_module(module).await?;
            }

            bar.run().await
        });

    if let Err(e) = res {
        println!("{e}");
        // TODO - handle error
    }
}
