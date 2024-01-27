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

        let result = match self.stream.next().await {
            Some(result) => result,
            None => return None,
        };

        let response = match result {
            Ok(response) => response,
            Err(error) => return Some(Err(Error(error))),
        };

        let mut events = Vec::new();
        for event in response.events() {
            events.push(event.clone().into());
        }

        self.events = events;
        let event = self.events[0].clone();

        self.index = 1;
        Some(Ok(event))
    }
}
