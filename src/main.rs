mod splitter;
use splitter::SplitterThread;

mod shared;
use shared::SharedState;

mod livesplit_socket;
use livesplit_socket::commands::WsCommand;
use livesplit_socket::thread::WsThread;
use livesplit_socket::timer::WebsocketTimer;

use std::{
    net::{IpAddr, TcpListener},
    path::PathBuf,
    sync::{Arc, RwLock},
    thread,
};

use anyhow::Result;
use clap::Parser;
use livesplit_auto_splitting::TimerState;
use log::{debug, info};

#[derive(Parser, Debug)]
#[command(about, long_about = None, arg_required_else_help(true))]
struct Args {
    /// Path to a settings file for the autosplitter (toml)
    #[arg(short, long)]
    settings: Option<PathBuf>,

    /// Websocket port
    #[arg(short, long, default_value_t = 9087)]
    port: u16,

    /// Websocket host
    #[arg(short = 'H', long, default_value = "0.0.0.0")]
    host: IpAddr,

    /// Path to the autosplitter wasm file
    wasm_path: PathBuf,
}

fn main() -> Result<()> {
    if std::env::var("RUST_LOG").is_err() {
        unsafe { std::env::set_var("RUST_LOG", "info") };
    }
    pretty_env_logger::init_timed();

    let args = Args::parse();
    debug!("Args: {:?}", args);

    let server = TcpListener::bind((args.host, args.port))?;
    info!("Listening on {:?}", server.local_addr());

    for (counter, stream) in server.incoming().enumerate() {
        let stream = stream?;
        info!("Accepting connection from {:?}", stream.peer_addr());
        let ws = tungstenite::accept(stream)?;
        ws.get_ref().set_nonblocking(true)?;

        let timer_state = Arc::new(RwLock::new(TimerState::NotRunning));

        let (mut ws, tx) = WsThread::new(ws, Arc::clone(&timer_state));
        let _ = ws.handle(WsCommand::GetCurrentState);

        let shared_state = SharedState::new(timer_state);
        let timer = WebsocketTimer::new(shared_state.clone(), tx);
        let state = SplitterThread::new(
            &args.wasm_path,
            args.settings.as_deref(),
            timer,
            shared_state,
        )?;

        thread::Builder::new()
            .name(format!("Websocket Handler {counter}"))
            .spawn(move || ws.run())
            .unwrap();

        thread::Builder::new()
            .name(format!("Auto Splitter Runtime {counter}"))
            .spawn(move || state.run())
            .unwrap();
    }

    Ok(())
}
