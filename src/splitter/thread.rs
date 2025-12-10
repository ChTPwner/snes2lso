use std::{fs, path::Path, thread, time::Instant};

use anyhow::Context;
use livesplit_auto_splitting::{AutoSplitter, Config, Runtime, settings};
use log::error;

use crate::{livesplit_socket::WebsocketTimer, shared::SharedState};

pub struct SplitterThread {
    splitter: AutoSplitter<WebsocketTimer>,
    state: SharedState,
}

impl SplitterThread {
    pub fn new(
        path: &Path,
        settings: Option<&Path>,
        timer: WebsocketTimer,
        state: SharedState,
    ) -> anyhow::Result<Self> {
        let module =
            fs::read(path).context("Failed loading the auto splitter from the file system.")?;

        let mut settings_map = settings::Map::new();
        if let Some(settings) = settings {
            SplitterThread::load_settings(settings, &mut settings_map)?;
        }

        let runtime = {
            let mut config = Config::default();
            config.debug_info = false;
            config.optimize = true;
            config.backtrace_details = false;
            Runtime::new(config)?
        };

        let module = runtime
            .compile(&module)
            .context("Failed loading the auto splitter.")?;

        let splitter = module
            .instantiate(timer, Some(settings_map), None)
            .context("Failed starting the auto splitter.")?;

        Ok(SplitterThread { splitter, state })
    }

    fn load_settings(file: &Path, settings_map: &mut settings::Map) -> anyhow::Result<()> {
        let settings = fs::read_to_string(file)?;
        let settings = toml::from_str::<toml::Table>(&settings)?;

        for (key, value) in settings {
            let value = match value {
                toml::Value::Boolean(value) => settings::Value::Bool(value),
                toml::Value::String(value) => settings::Value::String(value.into()),
                toml::Value::Integer(value) => settings::Value::I64(value),
                toml::Value::Float(value) => settings::Value::F64(value),
                _ => anyhow::bail!("Unsupported value type: {value:?}"),
            };

            settings_map.insert(key.into(), value);
        }

        Ok(())
    }

    pub fn run(self) {
        let mut next_tick = Instant::now();

        loop {
            let auto_splitter = &self.splitter;

            let mut auto_splitter_lock = auto_splitter.lock();
            // does the actual work
            let res = auto_splitter_lock.update();
            drop(auto_splitter_lock);

            if let Err(e) = res {
                error!("{:?}", e.context("Failed executing the auto splitter."));
            };

            if !self.state.alive() {
                break;
            }

            let tick_rate = auto_splitter.tick_rate();
            next_tick += tick_rate;

            let now = Instant::now();
            if let Some(sleep_time) = next_tick.checked_duration_since(now) {
                thread::sleep(sleep_time);
            } else {
                next_tick = now;
            }
        }
    }
}
