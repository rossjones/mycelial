pub mod destination;

use bacalhau::BacalhauPayload;
use section::Message as _Message;

pub type Message = _Message<BacalhauPayload>;
