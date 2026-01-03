use etcd_client::WatchStream;
use pyo3::pyclass;
use tokio::sync::Mutex;
use tokio_stream::StreamExt;

use crate::{error::PyClientError, watch_event::PyWatchEvent};

#[pyclass(name = "WatchEventStream")]
pub struct PyWatchEventStream {
    stream: Mutex<WatchStream>,
    events: Mutex<Vec<PyWatchEvent>>,
    index: Mutex<usize>,
    once: bool,
}

impl PyWatchEventStream {
    pub fn new(stream: WatchStream, once: bool) -> Self {
        Self {
            stream: Mutex::new(stream),
            events: Mutex::new(Vec::new()),
            index: Mutex::new(0),
            once,
        }
    }

    pub async fn next(&mut self) -> Option<Result<PyWatchEvent, PyClientError>> {
        let mut index = self.index.lock().await;
        if self.once && *index > 0 {
            return None;
        }

        let mut events = self.events.lock().await;
        if *index < events.len() {
            let event = events[*index].clone();
            *index += 1;
            return Some(Ok(event));
        }

        let mut stream = self.stream.lock().await;
        match stream.next().await {
            Some(Ok(response)) => {
                let response_events = response.events();
                for event in response_events {
                    events.push(event.clone().into());
                }

                if !events.is_empty() {
                    let event = events[*index].clone();
                    *index += 1;
                    Some(Ok(event))
                } else {
                    None
                }
            }
            Some(Err(error)) => Some(Err(PyClientError(error))),
            None => None,
        }
    }
}
