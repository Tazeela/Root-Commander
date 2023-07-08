use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

// Bluetooth messages are polled asynchronously by a seperate thread. This storage class
// takes the messages, holds them, and is accessible from both that thread and the main thread.
// TODO: Really want to use a RWLock here instead of a Mutex but CondVar doesnt support it.
//  If perf is an issue might need to re-evaluate
pub struct MessageStorage<T: Eq + Hash, V> {
    cond_var: Condvar,
    messages: Arc<Mutex<HashMap<T, V>>>,
}

impl<T: Eq + Hash, V> MessageStorage<T, V> {
    pub fn new() -> MessageStorage<T, V> {
        MessageStorage {
            cond_var: Condvar::new(),
            messages: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    // Wait for a message to be received
    // TODO: Instead of blocking make this into a future?
    pub fn wait_for_message(&self, key: T) -> Option<V> {
        // Check to see if the message has been received
        // Note that any message we receive will notify the cond_var so we might get a few false
        // positives before actually getting the message we want.
        loop {
            let mut messages = self.messages.lock().unwrap();

            // Try and remove the message
            let message_opt = messages.remove(&key);
            if message_opt.is_some() {
                // Message has been received, done waiting
                return Some(message_opt.unwrap());
            } else {
                // Otherwise wait until another message is received to try again
                // TODO: Make timeout configurable?
                let res = self
                    .cond_var
                    .wait_timeout(messages, Duration::from_secs(10));

                if res.is_err() {
                    return None;
                }
            }
        }
    }

    // Put a message in storage, and notify any waiters
    pub fn put_message(&self, key: T, message: V) {
        self.messages.lock().unwrap().insert(key, message);
        self.cond_var.notify_all();
    }
}

#[cfg(test)]
mod message_storage_tests {
    use super::MessageStorage;
    use std::thread;

    #[test]
    fn can_add_message_and_wait() {
        let mut message_count = 0;
        let storage: MessageStorage<u8, u8> = MessageStorage::new();

        thread::scope(|s| {
            s.spawn(|| {
                storage.put_message(0x01, 0x1A);
            });
            s.spawn(|| {
                let message = storage.wait_for_message(0x01);
                if message.unwrap() == 0x1A {
                    message_count += 1;
                }
            });
        });

        assert_eq!(1, message_count);
    }
}
