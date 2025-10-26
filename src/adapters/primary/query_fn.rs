use crate::adapters::secondary::{PromptType, SubprocessCLITransport};
use crate::application::Query;
use crate::domain::Message;
use crate::error::Result;
use crate::options::AgentOptions;
use futures::Stream;
use futures::StreamExt;
use tokio::sync::mpsc;

/// One-shot query function
/// Returns a stream of messages
pub async fn query(
    prompt: impl Into<String>,
    options: Option<AgentOptions>,
) -> Result<impl Stream<Item = Message>> {
    let options = options.unwrap_or_default();
    let prompt_str = prompt.into();

    let transport = Box::new(SubprocessCLITransport::new(
        PromptType::String(prompt_str),
        options,
    ));

    let (mut query, write_rx) = Query::new(transport, false, None, None);

    // Connect transport
    {
        let transport_arc = query.transport();
        let mut transport_lock = transport_arc.lock().await;
        transport_lock.connect().await?;
    }

    query.start(write_rx).await?;

    // Create a channel to forward messages
    let (tx, rx) = mpsc::unbounded_channel();

    // Spawn a task to forward messages from query to channel
    tokio::spawn(async move {
        let stream = query.receive_messages();
        futures::pin_mut!(stream);
        while let Some(msg) = stream.next().await {
            if tx.send(msg).is_err() {
                break;
            }
        }
    });

    Ok(tokio_stream::wrappers::UnboundedReceiverStream::new(rx))
}
