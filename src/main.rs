use clap::Parser;

use honeybee::bar::Bar;
use honeybee::config::Config;
use honeybee::errors::*;
use honeybee::utils::{get_config_path, read_toml_config};
use honeybee::CliArgs;

fn main() {
    let args = CliArgs::parse();

    let res = tokio::runtime::Builder::new_current_thread()
        .max_blocking_threads(args.num_threads)
        .enable_all()
        .build()
        .unwrap()
        .block_on(async move {
            let config_file =
                get_config_path(&args.config_file).error("Configuration file not found")?;
            let mut config: Config = read_toml_config(config_file)?;
            config.general.colors = !args.no_colors;
            let modules = std::mem::take(&mut config.modules);
            let mut bar = Bar::new(config.general);

            for module in modules {
                bar.add_module(module).await?;
            }

            bar.run(args.run_once).await
        });

    if let Err(e) = res {
        println!("{e}");
        // TODO - handle error
    }
}
