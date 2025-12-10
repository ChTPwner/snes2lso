use crate::shared::SharedState;

use crate::livesplit_socket::WsCommand;
use livesplit_auto_splitting::{LogLevel, Timer, TimerState, time};
use log::{error, info, trace};
use std::sync::mpsc::Sender;

pub struct WebsocketTimer {
    state: SharedState,
    rx: Sender<WsCommand>,
}

impl WebsocketTimer {
    pub fn new(state: SharedState, rx: Sender<WsCommand>) -> Self {
        Self { state, rx }
    }

    fn send(&self, cmd: WsCommand) {
        if let Err(e) = self.rx.send(cmd) {
            error!("Could not send command to the websocket: {e:?}");
            self.state.deadge();
        }
    }
}

impl Timer for WebsocketTimer {
    fn state(&self) -> TimerState {
        self.state.state()
    }

    fn start(&mut self) {
        trace!("Start");
        self.send(WsCommand::Start);
    }

    fn split(&mut self) {
        trace!("Split");
        self.send(WsCommand::Split);
    }

    fn skip_split(&mut self) {
        trace!("Skip split");
        self.send(WsCommand::SkipSplit);
    }

    fn undo_split(&mut self) {
        trace!("Undo split");
        self.send(WsCommand::UndoSplit);
    }

    fn reset(&mut self) {
        trace!("Reset");
        self.send(WsCommand::Reset);
    }

    fn set_game_time(&mut self, time: time::Duration) {
        trace!("Set game time to {time:?}");
        self.send(WsCommand::SetGameTime(time));
    }

    fn pause_game_time(&mut self) {
        trace!("Pause game time");
        self.send(WsCommand::PauseGameTime);
    }

    fn resume_game_time(&mut self) {
        trace!("Resume game time");
        self.send(WsCommand::ResumeGameTime);
    }

    fn set_variable(&mut self, key: &str, value: &str) {
        trace!("Set variable {key} = {value}");
        self.send(WsCommand::SetCustomVariable(key.into(), value.into()));
    }

    fn log_auto_splitter(&mut self, message: std::fmt::Arguments<'_>) {
        info!("Autosplitter: {message}");
    }

    fn log_runtime(&mut self, message: std::fmt::Arguments<'_>, log_level: LogLevel) {
        let level = match log_level {
            LogLevel::Trace => log::Level::Trace,
            LogLevel::Debug => log::Level::Debug,
            LogLevel::Info => log::Level::Info,
            LogLevel::Warning => log::Level::Warn,
            LogLevel::Error => log::Level::Error,
        };
        log::log!(level, "{message}");
    }
}
