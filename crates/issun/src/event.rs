//! Type-erased, buffered event bus for decoupled communication between systems.
//!
//! Events are double buffered per type: events published during frame `N` are
//! consumed in frame `N + 1` after the runner calls [`EventBus::dispatch`].

use std::any::{Any, TypeId};
use std::collections::HashMap;

/// Marker trait for types that can flow through the [`EventBus`].
///
/// Events must be cloneable and thread-safe because they are buffered and may
/// be read from multiple systems during a frame.
pub trait Event: Clone + Send + Sync + 'static {}

/// Event bus stored inside [`ResourceContext`](crate::context::ResourceContext).
///
/// Each event type has its own [`EventChannel`] that double-buffers events so
/// publishers and subscribers do not contend within the same frame.
pub struct EventBus {
    channels: HashMap<TypeId, Box<dyn EventChannelStorage>>,
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
        }
    }

    /// Publishes a new event.
    ///
    /// Events are queued for the next frame and become visible after
    /// [`EventBus::dispatch`] runs.
    pub fn publish<E>(&mut self, event: E)
    where
        E: Event,
    {
        let channel = self.channel_mut::<E>();
        channel.push(event);
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

    #[derive(Clone, Debug, PartialEq)]
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
