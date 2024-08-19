use std::{collections::BTreeSet, fmt::Display};

use anyhow::Result;
use btleplug::{
    api::{Central, CentralEvent, Characteristic, Manager as _, Peripheral, ScanFilter, WriteType},
    platform::Manager,
};
use clap::{Parser, ValueEnum};
use futures::{stream::BoxStream, StreamExt, TryStreamExt};
use uuid::{uuid, Uuid};

const CHARACTERISTIC_UUID: Uuid = uuid!("00001525-1212-efde-1523-785feabcd124");

macro_rules! guard {
    ($ex:expr, $else:expr) => {
        if let Some(x) = $ex {
            x
        } else {
            $else
        }
    };
}

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(value_enum)]
    command: Command,
    /// Base station names to control or show.
    /// If nothing is specified, it will scan endlessly and show or control all discovered base stations.
    names: Vec<String>,
}

#[derive(Clone, ValueEnum)]
enum Command {
    On,
    Sleep,
    Standby,
    Scan,
}

enum PowerState {
    Sleep,
    Booting,
    Standby,
    On,
    Unknown(u8),
}

impl From<u8> for PowerState {
    fn from(byte: u8) -> Self {
        match byte {
            0x00 => PowerState::Sleep,
            0x01 | 0x08 | 0x09 => PowerState::Booting,
            0x02 => PowerState::Standby,
            0x0b => PowerState::On,
            byte => PowerState::Unknown(byte),
        }
    }
}

impl From<PowerState> for u8 {
    fn from(state: PowerState) -> Self {
        match state {
            PowerState::Sleep => 0x00,
            PowerState::Booting => 0x01,
            PowerState::Standby => 0x02,
            PowerState::On => 0x01,
            PowerState::Unknown(byte) => byte,
        }
    }
}

impl Display for PowerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PowerState::Sleep => write!(f, "SLEEP"),
            PowerState::Standby => write!(f, "STANDBY"),
            PowerState::Booting => write!(f, "BOOTING"),
            PowerState::On => write!(f, "ON"),
            PowerState::Unknown(byte) => write!(f, "UNKNOWN(0x{:02x})", byte),
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let manager = Manager::new().await?;
    let adapters = manager.adapters().await?;
    let central = adapters.into_iter().next().unwrap();
    let filter = Filter::new(cli.names);
    scan(&central, cli.command, filter).await?;
    Ok(())
}

struct Filter {
    names: Option<BTreeSet<String>>,
}

impl Filter {
    fn new(names: Vec<String>) -> Self {
        Self {
            names: if names.is_empty() {
                None
            } else {
                Some(BTreeSet::from_iter(names.into_iter()))
            },
        }
    }

    fn is_completed(&self) -> bool {
        if let Some(set) = self.names.as_ref() {
            set.is_empty()
        } else {
            false
        }
    }

    fn is_matched(&mut self, name: &str) -> bool {
        if let Some(set) = self.names.as_mut() {
            set.remove(name)
        } else {
            true
        }
    }
}

async fn scan(central: &impl Central, command: Command, mut filter: Filter) -> Result<()> {
    let mut stream = discover(central).await?;
    while !filter.is_completed() {
        let lh = guard!(stream.try_next().await?, break);
        if filter.is_matched(&lh.name) {
            let bytes = lh.peripheral.read(&lh.characteristic).await?;
            let current_state: PowerState = (*guard!(bytes.get(0), continue)).into();
            let next_state = match command {
                Command::Scan => {
                    println!("{}: {}", lh.name, current_state);
                    continue;
                }
                Command::On => PowerState::On,
                Command::Standby => PowerState::Standby,
                Command::Sleep => PowerState::Sleep,
            };
            println!("{}: {} -> {}", lh.name, current_state, next_state);
            lh.peripheral
                .write(
                    &lh.characteristic,
                    &[next_state.into()],
                    WriteType::WithoutResponse,
                )
                .await?;
        }
    }
    Ok(())
}

struct Lighthouse<P> {
    name: String,
    peripheral: P,
    characteristic: Characteristic,
}

async fn discover<C: Central>(central: &C) -> Result<BoxStream<Result<Lighthouse<C::Peripheral>>>> {
    central.start_scan(ScanFilter::default()).await?;
    let events = central.events().await?;
    Ok(events
        .filter_map(|ev| async {
            match ev {
                CentralEvent::DeviceDiscovered(id) | CentralEvent::DeviceUpdated(id) => Some(id),
                _ => None,
            }
        })
        .map(anyhow::Ok)
        .and_then(move |id| async move { Ok(central.peripheral(&id).await?) })
        .try_filter_map(move |p| async move {
            let props = guard!(p.properties().await?, return Ok(None));
            if !props.manufacturer_data.contains_key(&0x055d) {
                return Ok(None);
            }
            let local_name = guard!(props.local_name, return Ok(None));
            p.connect().await?;
            p.discover_services().await?;
            p.disconnect().await?;
            let characteristic = p
                .characteristics()
                .into_iter()
                .find(|ch| ch.uuid == CHARACTERISTIC_UUID);
            let characteristic = guard!(characteristic, return Ok(None));
            Ok(Some(Lighthouse {
                name: local_name,
                peripheral: p,
                characteristic,
            }))
        })
        .boxed())
}
