use crate::livesplit_socket::events::CommandResult;
use crate::livesplit_socket::events::State;
use crate::livesplit_socket::events::{Event, Response};
use std::{
    net::TcpStream,
    ops::ControlFlow,
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, RwLock,
    },
    time::Duration,
};

use anyhow::Result;
use livesplit_auto_splitting::{time, TimerState};
use log::{error, info, trace};
use tungstenite::{Message, Utf8Bytes, WebSocket};

use crate::livesplit_socket::WsCommand;
macro_rules! cmd {
    ($command:literal) => {
        Message::Text(Utf8Bytes::from_static(concat!(
            "{\"command\":\"",
            $command,
            "\"}"
        )))
    };
}

pub struct WsThread {
    ws: WebSocket<TcpStream>,
    rx: Receiver<WsCommand>,
    timer_state: Arc<RwLock<TimerState>>,
}

impl WsThread {
    pub fn new(
        ws: WebSocket<TcpStream>,
        timer_state: Arc<RwLock<TimerState>>,
    ) -> (Self, Sender<WsCommand>) {
        let (tx, rx) = mpsc::channel();
        (
            Self {
                ws,
                rx,
                timer_state,
            },
            tx,
        )
    }

    const START: Message = cmd!("start");
    const SPLIT: Message = cmd!("split");
    const RESET: Message = cmd!("reset");
    const UNDO_SPLIT: Message = cmd!("undoSplit");
    const SKIP_SPLIT: Message = cmd!("skipSplit");
    const PAUSE_GAME_TIME: Message = cmd!("pauseGameTime");
    const RESUME_GAME_TIME: Message = cmd!("resumeGameTime");
    const GET_CURRENT_STATE: Message = cmd!("getCurrentState");

    fn set_game_time(time: time::Duration) -> Message {
        Message::text(format!(
            "{{\"command\":\"setGameTime\",\"time\":\"{}\"}}",
            time.whole_seconds()
        ))
    }

    fn set_custom_variable(key: &str, value: &str) -> Message {
        Message::text(format!(
            "{{\"command\":\"setCustomVariable\",\"key\":\"{}\",\"value\":\"{}\"}}",
            key, value
        ))
    }

    fn parse_response(text: &str) -> Result<CommandResult> {
        let response = serde_json::from_str::<CommandResult>(text)?;
        trace!("Websocket response: {response:?}");
        Ok(response)
    }

    pub fn run(mut self) {
        loop {
            let cb = match self.rx.recv_timeout(Duration::from_millis(10)) {
                Ok(cmd) => self.handle(cmd),
                Err(mpsc::RecvTimeoutError::Timeout) => self.read(),
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    info!("Sender disconnected");
                    ControlFlow::Break(())
                }
            };

            match cb {
                ControlFlow::Continue(()) => {}
                ControlFlow::Break(()) => break,
            };
        }
    }

    pub fn handle(&mut self, cmd: WsCommand) -> ControlFlow<()> {
        macro_rules! send {
            ($msg:expr) => {
                if let Err(e) = self.ws.send($msg) {
                    error!("Websocket Write Error: {e:?}");
                }
            };
        }

        match &cmd {
            WsCommand::Start => send!(Self::START),
            WsCommand::Split => send!(Self::SPLIT),
            WsCommand::Reset => send!(Self::RESET),
            WsCommand::UndoSplit => send!(Self::UNDO_SPLIT),
            WsCommand::SkipSplit => send!(Self::SKIP_SPLIT),
            WsCommand::SetGameTime(time) => send!(Self::set_game_time(*time)),
            WsCommand::PauseGameTime => send!(Self::PAUSE_GAME_TIME),
            WsCommand::ResumeGameTime => send!(Self::RESUME_GAME_TIME),
            WsCommand::SetCustomVariable(key, value) => {
                send!(Self::set_custom_variable(key, value))
            }
            WsCommand::GetCurrentState => send!(Self::GET_CURRENT_STATE),
        }

        self.read()
    }

    fn read(&mut self) -> ControlFlow<()> {
        let msg = match self.ws.read() {
            Ok(msg) => msg,
            Err(tungstenite::Error::Io(e)) if e.kind() == std::io::ErrorKind::WouldBlock => {
                return ControlFlow::Continue(());
            }
            Err(e) => {
                error!("Websocket Read Error: {e:?}");
                return ControlFlow::Break(());
            }
        };
        trace!("Websocket message: {msg:?}");

        let msg = match msg {
            Message::Text(ref utf8_bytes) => utf8_bytes.as_str(),
            Message::Binary(ref bytes) => std::str::from_utf8(bytes).unwrap(),
            Message::Close(_) => {
                info!("Other side hang up, stopping");
                return ControlFlow::Break(());
            }
            Message::Ping(msg) => {
                info!("Ping received, sending pong");
                let _ = self.ws.send(Message::Pong(msg));
                return ControlFlow::Continue(());
            }
            otherwise => {
                error!("Not a text message: {otherwise:?}");
                return ControlFlow::Break(());
            }
        };

        let msg = match Self::parse_response(msg) {
            Ok(msg) => msg,
            Err(e) => {
                error!("Websocket Parse Error: {e:?}");
                return ControlFlow::Continue(());
            }
        };
        trace!("Websocket message parsed: {msg:?}");

        let state = match msg {
            CommandResult::Success(Response::State(state)) => Some(match state {
                State::NotRunning => TimerState::NotRunning,
                State::Running(_) => TimerState::Running,
                State::Paused(_) => TimerState::Paused,
                State::Ended => TimerState::Ended,
            }),
            CommandResult::Event(Event::Started) => Some(TimerState::Running),
            CommandResult::Event(Event::Paused) => Some(TimerState::Paused),
            CommandResult::Event(Event::Resumed) => Some(TimerState::Running),
            CommandResult::Event(Event::Finished) => Some(TimerState::Ended),
            CommandResult::Event(Event::Reset) => Some(TimerState::NotRunning),
            CommandResult::Success(_) | CommandResult::Event(_) | CommandResult::Error(_) => None,
        };

        if let Some(state) = state {
            info!("Changing timer state to {state:?}");
            let mut guard = self.timer_state.write().unwrap_or_else(|e| e.into_inner());
            *guard = state;
        }

        ControlFlow::Continue(())
    }
}
