#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]
#![cfg_attr(feature = "clippy", deny(clippy))]

#[macro_use]
extern crate clap;

#[macro_use]
extern crate log;

#[macro_use]
extern crate serde_json;

use log4rs;
use tokio;
use xrl;

mod core;
mod widgets;
use xdg::BaseDirectories;

use failure::Error;
use futures::{future, Future, Stream};
use log::LevelFilter;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Logger, Root};
use xrl::spawn;

use crate::core::{Command, Tui, TuiServiceBuilder, KeybindingConfig};

fn configure_logs(logfile: &str) {
    let tui = FileAppender::builder().build(logfile).unwrap();
    let rpc = FileAppender::builder()
        .build(&format!("{}.rpc", logfile))
        .unwrap();
    let config = Config::builder()
        .appender(Appender::builder().build("tui", Box::new(tui)))
        .appender(Appender::builder().build("rpc", Box::new(rpc)))
        .logger(
            Logger::builder()
                .appender("tui")
                .additive(false)
                .build("xi_tui", LevelFilter::Debug),
        )
        .logger(
            Logger::builder()
                .appender("tui")
                .additive(false)
                .build("xrl", LevelFilter::Info),
        )
        .logger(
            Logger::builder()
                .appender("rpc")
                .additive(false)
                .build("xrl::protocol::codec", LevelFilter::Trace),
        )
        .build(Root::builder().appender("tui").build(LevelFilter::Info))
        .unwrap();
    let _ = log4rs::init_config(config).unwrap();
}

fn main() {
    if let Err(ref e) = run() {
        use std::io::Write;
        let stderr = &mut ::std::io::stderr();

        writeln!(stderr, "error: {}", e).unwrap();
        error!("error: {}", e);

        writeln!(stderr, "caused by: {}", e.as_fail()).unwrap();
        error!("error: {}", e);

        writeln!(stderr, "backtrace: {:?}", e.backtrace()).unwrap();
        error!("error: {}", e);

        ::std::process::exit(1);
    }
}

fn run() -> Result<(), Error> {
    let xi = clap_app!(
        xi =>
        (about: "The Xi Editor")
        (@arg core: -c --core +takes_value "Specify binary to use for the backend")
        (@arg logfile: -l --logfile +takes_value "Log file location")
        (@arg file: +required "File to edit"));

    let matches = xi.get_matches();
    if let Some(logfile) = matches.value_of("logfile") {
        configure_logs(logfile);
    }

    // let configfile = std::path::Path::new("./configs/Default (Linux).sublime-keymap").to_owned();
    // let keybindings = KeybindingConfig::parse(&configfile).map_err(Error::from_boxed_compat)?;
    let keybindings = KeybindingConfig::parse().map_err(Error::from_boxed_compat)?;

    tokio::run(future::lazy(move || {
        info!("starting xi-core");
        let (tui_service_builder, core_events_rx) = TuiServiceBuilder::new();
        let (client, core_stderr) = spawn(
            matches.value_of("core").unwrap_or("xi-core"),
            tui_service_builder,
        );

        info!("starting logging xi-core errors");
        tokio::spawn(
            core_stderr
                .for_each(|msg| {
                    error!("core: {}", msg);
                    Ok(())
                })
                .map_err(|_| ()),
        );

        tokio::spawn(future::lazy(move || {
            let conf_dir = BaseDirectories::with_prefix("xi")
                .ok()
                .and_then(|dirs| Some(dirs.get_config_home().to_string_lossy().into_owned()));

            let client_clone = client.clone();
            client
                .client_started(conf_dir.as_ref().map(|dir| &**dir), None)
                .map_err(|e| error!("failed to send \"client_started\" {:?}", e))
                .and_then(move |_| {
                    info!("initializing the TUI");
                    let mut tui = Tui::new(client_clone, core_events_rx, keybindings)
                        .expect("failed to initialize the TUI");
                    tui.run_command(Command::Open(
                        matches.value_of("file").map(ToString::to_string),
                    ));
                    tui.run_command(Command::SetTheme("Solarized (dark)".into()));
                    tui.map_err(|e| error!("TUI exited with an error: {:?}", e))
                })
        }));
        Ok(())
    }));
    Ok(())
}
