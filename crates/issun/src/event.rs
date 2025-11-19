//! Type-erased, buffered event bus for decoupled communication between systems.
//!
//! Events are double buffered per type: events published during frame `N` are
//! consumed in frame `N + 1` after the runner calls [`EventBus::dispatch`].

use std::any::{Any, TypeId};
use std::collections::HashMap;

#[cfg(feature = "network")]
use crate::network::{NetworkMetadata, NetworkScope};

/// Marker trait for types that can flow through the [`EventBus`].
///
/// Events must be cloneable and thread-safe because they are buffered and may
/// be read from multiple systems during a frame.
pub trait Event: Clone + Send + Sync + 'static {
    /// Check if this event should be transmitted over network
    #[cfg(feature = "network")]
    fn is_networked() -> bool {
        false
    }

    /// Get the network scope for this event
    #[cfg(feature = "network")]
    fn network_scope() -> NetworkScope {
        NetworkScope::default()
    }
}

/// Event bus stored inside [`ResourceContext`](crate::context::ResourceContext).
///
/// Each event type has its own [`EventChannel`] that double-buffers events so
/// publishers and subscribers do not contend within the same frame.
pub struct EventBus {
    channels: HashMap<TypeId, Box<dyn EventChannelStorage>>,

    #[cfg(feature = "network")]
    network: Option<NetworkState>,
}

#[cfg(feature = "network")]
struct NetworkState {
    backend: std::sync::Arc<dyn crate::network::NetworkBackend>,
    tx: tokio::sync::mpsc::Sender<NetworkTask>,
    sequence: std::sync::atomic::AtomicU64,
    current_metadata: Option<NetworkMetadata>,
}

#[cfg(feature = "network")]
enum NetworkTask {
    Send(Vec<u8>), // Serialized RawNetworkEvent
    Shutdown,
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl EventBus {
    /// Creates an empty event bus.
    pub fn new() -> Self {
        Self {
            channels: HashMap::new(),
            #[cfg(feature = "network")]
            network: None,
        }
    }

    /// Publishes a new event.
    ///
    /// Events are queued for the next frame and become visible after
    /// [`EventBus::dispatch`] runs.
    ///
    /// If the event is marked as networked and network is enabled, the event
    /// will also be transmitted to remote nodes.
    pub fn publish<E>(&mut self, event: E)
    where
        E: Event + serde::Serialize,
    {
        // Always perform local dispatch
        let channel = self.channel_mut::<E>();
        channel.push(event.clone());

        // If networked, send to network backend
        #[cfg(feature = "network")]
        if E::is_networked() {
            if let Some(ref mut net) = self.network {
                let sequence = net
                    .sequence
                    .fetch_add(1, std::sync::atomic::Ordering::SeqCst);

                let metadata = NetworkMetadata::new(net.backend.node_id(), sequence);
                let scope = E::network_scope();

                // Create RawNetworkEvent
                let raw_event = crate::network::backend::RawNetworkEvent {
                    metadata,
                    scope,
                    type_name: std::any::type_name::<E>().to_string(),
                    payload: bincode::serialize(&event).unwrap_or_default(),
                };

                // Queue for async send
                if let Ok(serialized) = bincode::serialize(&raw_event) {
                    let _ = net.tx.try_send(NetworkTask::Send(serialized));
                }
            }
        }
    }

    /// Returns a reader over events of type `E` from the previous frame.
    ///
    /// Readers borrow the bus immutably for the lifetime of the reader to
    /// prevent the channel from being mutated while iterating.
    pub fn reader<E>(&mut self) -> EventReader<'_, E>
    where
        E: Event,
    {
        let channel = self.channel_mut::<E>();
        EventReader {
            events: channel.read(),
            cursor: 0,
        }
    }

    /// Advances all event channels by swapping their buffers.
    ///
    /// This should be invoked once per frame (typically by the runner). After
    /// dispatching, events published this frame become visible in the next one.
    pub fn dispatch(&mut self) {
        for channel in self.channels.values_mut() {
            channel.swap_buffers();
        }
    }

    fn channel_mut<E>(&mut self) -> &mut EventChannel<E>
    where
        E: Event,
    {
        let entry = self
            .channels
            .entry(TypeId::of::<E>())
            .or_insert_with(|| Box::new(EventChannel::<E>::new()));

        entry
            .as_any_mut()
            .downcast_mut::<EventChannel<E>>()
            .expect("Stored channel type mismatch")
    }

    /// Enable network support (network feature only)
    #[cfg(feature = "network")]
    pub fn with_network(mut self, backend: impl crate::network::NetworkBackend) -> Self {
        use std::sync::Arc;
        use tokio::sync::mpsc;

        let backend = Arc::new(backend);
        let (tx, rx) = mpsc::channel(1000);

        // Spawn background worker for sending events
        let backend_clone = backend.clone();
        tokio::spawn(async move {
            network_send_worker(rx, backend_clone).await;
        });

        self.network = Some(NetworkState {
            backend,
            tx,
            sequence: std::sync::atomic::AtomicU64::new(0),
            current_metadata: None,
        });

        self
    }

    /// Get metadata of the currently processing networked event
    #[cfg(feature = "network")]
    pub fn current_metadata(&self) -> Option<&NetworkMetadata> {
        self.network
            .as_ref()
            .and_then(|n| n.current_metadata.as_ref())
    }

    /// Check if network is enabled
    #[cfg(feature = "network")]
    pub fn is_networked(&self) -> bool {
        self.network.is_some()
    }
}

/// Background worker for sending network events
#[cfg(feature = "network")]
async fn network_send_worker(
    mut rx: tokio::sync::mpsc::Receiver<NetworkTask>,
    backend: std::sync::Arc<dyn crate::network::NetworkBackend>,
) {
    use crate::network::backend::RawNetworkEvent;

    while let Some(task) = rx.recv().await {
        match task {
            NetworkTask::Send(data) => {
                // Deserialize and send
                if let Ok(event) = bincode::deserialize::<RawNetworkEvent>(&data) {
                    if let Err(e) = backend.send(event).await {
                        eprintln!("Failed to send network event: {:?}", e);
                    }
                }
            }
            NetworkTask::Shutdown => break,
        }
    }
}

/// Event reader that iterates over events published in the previous frame.
pub struct EventReader<'a, E>
where
    E: Event,
{
    events: &'a [E],
    cursor: usize,
}

impl<'a, E> EventReader<'a, E>
where
    E: Event,
{
    /// Returns an iterator over the unread events.
    pub fn iter(&self) -> impl Iterator<Item = &E> {
        self.events[self.cursor..].iter()
    }

    /// Number of unread events.
    pub fn len(&self) -> usize {
        self.events.len().saturating_sub(self.cursor)
    }

    /// Returns true if there are no unread events.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Internal event channel for a specific event type `E`.
///
/// `a` is the write buffer; `b` is the read buffer.
struct EventChannel<E>
where
    E: Event,
{
    a: Vec<E>,
    b: Vec<E>,
}

impl<E> EventChannel<E>
where
    E: Event,
{
    fn new() -> Self {
        Self {
            a: Vec::new(),
            b: Vec::new(),
        }
    }

    fn push(&mut self, event: E) {
        self.a.push(event);
    }

    fn read(&self) -> &[E] {
        &self.b
    }

    fn swap_buffers(&mut self) {
        std::mem::swap(&mut self.a, &mut self.b);
        self.a.clear();
    }
}

trait EventChannelStorage: Any + Send + Sync {
    fn swap_buffers(&mut self);
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<E> EventChannelStorage for EventChannel<E>
where
    E: Event,
{
    fn swap_buffers(&mut self) {
        EventChannel::swap_buffers(self);
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Convenience macro for collecting events from an EventBus into a Vec.
///
/// # Examples
///
/// ```ignore
/// use issun::event::{EventBus, collect_events};
///
/// let mut bus = EventBus::new();
/// let events = collect_events!(bus, MyEvent);
/// ```
#[macro_export]
macro_rules! collect_events {
    ($bus:expr, $event_type:ty) => {{
        $bus.reader::<$event_type>()
            .iter()
            .cloned()
            .collect::<Vec<$event_type>>()
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
    struct Damage(u32);

    impl Event for Damage {}

    #[test]
    fn publish_requires_dispatch() {
        let mut bus = EventBus::new();
        bus.publish(Damage(10));

        // Events are not visible until dispatch happens.
        let reader = bus.reader::<Damage>();
        assert!(reader.is_empty());

        bus.dispatch();
        let reader = bus.reader::<Damage>();
        assert_eq!(reader.len(), 1);
        assert_eq!(reader.iter().next(), Some(&Damage(10)));
    }

    #[test]
    fn multiple_dispatch_cycles() {
        let mut bus = EventBus::new();

        // Frame 0 publish
        bus.publish(Damage(1));
        bus.dispatch();
        let reader = bus.reader::<Damage>();
        assert_eq!(reader.iter().map(|d| d.0).collect::<Vec<_>>(), vec![1]);

        // Frame 1 publish multiple
        bus.publish(Damage(2));
        bus.publish(Damage(3));
        bus.dispatch();
        let reader = bus.reader::<Damage>();
        assert_eq!(reader.iter().map(|d| d.0).collect::<Vec<_>>(), vec![2, 3]);

        // Ensure old events cleared
        bus.dispatch();
        let reader = bus.reader::<Damage>();
        assert!(reader.is_empty());
    }
}
