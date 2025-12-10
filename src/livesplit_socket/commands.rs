use livesplit_auto_splitting::time;

#[derive(Debug, Clone)]
pub enum WsCommand {
    Start,
    Split,
    Reset,
    UndoSplit,
    SkipSplit,
    SetGameTime(time::Duration),
    PauseGameTime,
    ResumeGameTime,
    SetCustomVariable(Box<str>, Box<str>),
    GetCurrentState,
}
