use etcd_client::WatchStream;
use pyo3::pyclass;
use tokio_stream::StreamExt;

use crate::{error::Error, event::PyEvent};

#[pyclass(name = "EventStream")]
pub struct PyEventStream {
    stream: WatchStream,
    events: Vec<PyEvent>,
    index: usize,
}

impl PyEventStream {
    pub fn new(stream: WatchStream) -> Self {
        Self {
            stream,
            events: Vec::new(),
            index: 0,
        }
    }

    pub async fn next(&mut self) -> Option<Result<PyEvent, Error>> {
        if self.index < self.events.len() {
            let event = self.events[self.index].clone();
            self.index += 1;
            return Some(Ok(event));
        }

        match self.stream.next().await {
            Some(Ok(response)) => {
                let events = response.events();
                for event in events {
                    self.events.push(event.clone().into());
                }

                if self.events.len() > 0 {
                    let event = self.events[self.index].clone();
                    self.index += 1;
                    Some(Ok(event))
                } else {
                    None
                }
            }
            Some(Err(error)) => {
                Some(Err(Error(error)))
            }
            None => None
        }
    }
}
