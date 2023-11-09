//! Bacalhau Destination Section
//!
//! Receives messages from source sections, and after extracting
//! information from the RecordBatch, posts it to the configured
//! endpoint, to be processed by an external process.

use crate::api::*;
use crate::jobstore::JobStore;
use crate::BacalhauPayload;

use super::{Message, StdError};
use std::pin::{pin, Pin};

use futures::{Future, FutureExt, Sink, SinkExt, Stream, StreamExt};
use reqwest;
use section::{Command, Section, SectionChannel};

#[derive(Debug)]
pub struct Bacalhau {
    pub job: String,
    pub jobstore: JobStore,
}

impl Bacalhau {
    pub fn new(job: impl Into<String>, jobstore: impl Into<String>) -> Self {
        Self {
            job: job.into(),
            jobstore: JobStore::new(jobstore).expect("should be able to create jobstore"),
        }
    }

    pub async fn submit_job(&self, _payload: &BacalhauPayload) -> Result<(), StdError> {
        Ok(())
    }

    pub async fn enter_loop<Input, Output, SectionChan>(
        self,
        input: Input,
        output: Output,
        mut section_chan: SectionChan,
    ) -> Result<(), StdError>
    where
        Input: Stream<Item = Message> + Send + 'static,
        Output: Sink<Message, Error = StdError> + Send + 'static,
        SectionChan: SectionChannel + Send + Sync + 'static,
    {
        let mut input = pin!(input.fuse());
        let mut output = pin!(output);
        loop {
            futures::select_biased! {
                cmd = section_chan.recv().fuse() => {
                    if let Command::Stop = cmd? {
                        return Ok(())
                    }
                }
                msg = input.next() => {
                    let mut msg = match msg {
                        Some(msg) => msg,
                        None => Err("input stream closed")?
                    };
                    msg.ack().await;

                    let payload = &msg.payload;
                    let origin = &msg.origin;
                    self.submit_job(&payload).await?;

                    section_chan.log(&format!("Message from '{:?}' received! {:?}", origin, payload)).await?;
                    output.send(msg).await?;
                },
            }
        }
    }
}

impl<Input, Output, SectionChan> Section<Input, Output, SectionChan> for Bacalhau
where
    Input: Stream<Item = Message> + Send + 'static,
    Output: Sink<Message, Error = StdError> + Send + 'static,
    SectionChan: SectionChannel + Send + 'static,
{
    type Error = StdError;
    type Future = Pin<Box<dyn Future<Output = Result<(), Self::Error>> + Send + 'static>>;

    fn start(self, input: Input, output: Output, command: SectionChan) -> Self::Future {
        Box::pin(async move { self.enter_loop(input, output, command).await })
    }
}
