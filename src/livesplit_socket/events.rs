#[derive(Copy, Clone, Debug, PartialEq, Eq, serde_derive::Deserialize)]
pub enum EventError {
    Unsupported = 0,
    Busy = 1,
    RunAlreadyInProgress = 2,
    NoRunInProgress = 3,
    RunFinished = 4,
    NegativeTime = 5,
    CantSkipLastSplit = 6,
    CantUndoFirstSplit = 7,
    AlreadyPaused = 8,
    NotPaused = 9,
    ComparisonDoesntExist = 10,
    GameTimeAlreadyInitialized = 11,
    GameTimeAlreadyPaused = 12,
    GameTimeNotPaused = 13,
    CouldNotParseTime = 14,
    TimerPaused = 15,
    RunnerDecidedAgainstReset = 16,
    #[serde(other)]
    Unknown,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, serde_derive::Deserialize)]
pub enum Event {
    Started = 0,
    Splitted = 1,
    Finished = 2,
    Reset = 3,
    SplitUndone = 4,
    SplitSkipped = 5,
    Paused = 6,
    Resumed = 7,
    PausesUndone = 8,
    PausesUndoneAndResumed = 9,
    ComparisonChanged = 10,
    TimingMethodChanged = 11,
    GameTimeInitialized = 12,
    GameTimeSet = 13,
    GameTimePaused = 14,
    GameTimeResumed = 15,
    LoadingTimesSet = 16,
    CustomVariableSet = 17,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, serde_derive::Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(unused)]
pub enum CommandResult {
    Success(Response),
    Error(Error),
    Event(Event),
}

#[derive(Debug, serde_derive::Deserialize)]
#[serde(untagged)]
#[allow(unused)]
pub enum Response {
    None,
    String(String),
    State(State),
}

#[derive(Debug, serde_derive::Deserialize)]
#[serde(tag = "state", content = "index")]
#[allow(unused)]
pub enum State {
    NotRunning,
    Running(usize),
    Paused(usize),
    Ended,
}

#[derive(Debug, serde_derive::Deserialize)]
#[serde(tag = "code")]
#[allow(unused)]
pub enum Error {
    InvalidCommand {
        message: String,
    },
    InvalidIndex,
    #[serde(untagged)]
    Timer {
        code: EventError,
    },
}
