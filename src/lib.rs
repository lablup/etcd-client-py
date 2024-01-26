mod client;
mod communicator;
mod error;
mod event;
mod stream;
mod utils;
mod watch;

use client::Client;
use communicator::Communicator;
use event::Event;
use event::EventType;
use pyo3::prelude::*;
use watch::Watch;

#[pymodule]
fn etcd_client(_py: Python<'_>, module: &PyModule) -> PyResult<()> {
    module.add_class::<Client>()?;
    module.add_class::<Event>()?;
    module.add_class::<EventType>()?;
    module.add_class::<Communicator>()?;
    module.add_class::<Watch>()?;
    Ok(())
}
