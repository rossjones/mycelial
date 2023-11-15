//! Bacalhau Destination Section
//!
//! Receives messages from source sections, and after extracting
//! information from the RecordBatch, posts it to the configured
//! endpoint, to be processed by an external process.

use crate::api::submit;
use crate::jobstore::JobStore;
use crate::BacalhauPayload;

use super::{Message, StdError};
use std::{
    collections::HashMap,
    pin::{pin, Pin},
};

use futures::{Future, FutureExt, Sink, SinkExt, Stream, StreamExt};
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

    pub async fn submit_job(&self, payload: &BacalhauPayload) -> Result<String, StdError> {
        // We'll either get an Err from the call to render, or we'll get either
        // an Ok or Err from the call to submit.
        match self.jobstore.render(self.job.clone(), &payload.data) {
            Ok(output) => submit(&output).await,
            Err(m) => Err(m),
        }
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
                    println!("Ta-Da");
                    let mut msg = match msg {
                        Some(msg) => msg,
                        None => Err("input stream closed")?
                    };
                    msg.ack().await;

                    println!("ACKed message");

                    let payload = &msg.payload;
                    let origin = &msg.origin;
                    self.submit_job(payload).await?;

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

#[cfg(test)]
mod test {
    use super::*;
    use stub::Stub;

    use section::dummy::DummySectionChannel;
    use tokio::sync::mpsc::{Receiver, Sender};
    use tokio::time::{sleep, Duration};
    use tokio_stream::wrappers::ReceiverStream;
    use tokio_util::sync::PollSender;

    pub fn channel<T>(buf_size: usize) -> (PollSender<T>, ReceiverStream<T>)
    where
        T: Send + 'static,
    {
        let (tx, rx): (Sender<T>, Receiver<T>) = tokio::sync::mpsc::channel(buf_size);
        (PollSender::new(tx), ReceiverStream::new(rx))
    }

    #[tokio::test]
    async fn test_trigger() -> Result<(), StdError> {
        let jobstore: std::path::PathBuf =
            [env!("CARGO_MANIFEST_DIR"), "testdata"].iter().collect();
        let bac_dest = Bacalhau::new("process", jobstore.to_str().unwrap());

        let (mut tx, input) = channel::<Message>(1);

        let output = Stub::<Message, StdError>::new();
        let output = output.sink_map_err(|_| "chan closed".into());

        let section_chan = DummySectionChannel::new();

        let section = bac_dest.start(input, output, section_chan);
        let handle = tokio::spawn(section);

        let payload = BacalhauPayload {
            data: HashMap::new(),
        };

        futures::future::poll_fn(|cx| tx.poll_reserve(cx))
            .await
            .unwrap();

        tx.send_item(Message::new("test", payload, None)).unwrap();
        sleep(Duration::from_millis(100)).await;

        handle.abort();
        Ok(())
    }
}
