use etcd_client::WatchStream;
use tokio_stream::StreamExt;

use crate::{error::Error, event::Event};

pub struct Stream {
    stream: WatchStream,
    events: Vec<Event>,
    index: usize,
}

impl Stream {
    pub fn new(stream: WatchStream) -> Self {
        Self {
            stream,
            events: Vec::new(),
            index: 0,
        }
    }

    pub async fn next(&mut self) -> Option<Result<Event, Error>> {
        if self.index < self.events.len() {
            let event = self.events[self.index].clone();
            self.index += 1;
            return Some(Ok(event));
        }
        let option = self.stream.next().await;
        let result = match option {
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
