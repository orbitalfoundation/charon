
use crossbeam::channel::*;
use std::sync::Arc;
use std::sync::Mutex;

pub type SID = u64;

///
/// Message: this is an enumeration of messages that a service can send or receive
///

#[derive(Clone)]
pub enum Message {
    Share(Arc<Mutex<Box<[u32;921600]>>>),

    // register a new channel that can receive traffic
    Channel(SID,String,Sender<Message>),

    // listen to any traffic matching a string
    Subscribe(SID,String),
    Unsubscribe(SID,String),

    // Broker Goto - TODO for now special traffic directed at broker is special later perhaps just use ordinary events?
    // BrokerGoto(String),

    // Send an event to any traffic matching a string
    Event(String,String),

    // Dynamically build a service at runtime in the broker (not used right now)
    // Add(ServiceBuilder),

    // TODO examine - what i really want to do is send an actual trait instance...
    //Add2(&Serviceable),
    //Add2(Box<&Serviceable>),
    //AddInstance(Box<dyn Serviceable>),
    // https://www.reddit.com/r/rust/comments/7q3bz8/trait_object_with_clone/
    // https://www.reddit.com/r/rust/comments/8q0602/a_generic_trait_for_cloning_boxed_trait_objects/

}

///
/// Servicable: this is the definition of something that is capable of being a service
///

pub trait Serviceable: ServiceableClone {
    fn name(&self) -> &str;
    fn stop(&self);
    fn start(&self, name: String, sid: SID, send: Sender<Message>, recv: Receiver<Message> );
}

pub trait ServiceableClone {
    fn clone_box(&self) -> Box<dyn Serviceable>;
}

impl<T> ServiceableClone for T
    where T: 'static + Serviceable + Clone
{
    fn clone_box(&self) -> Box<dyn Serviceable> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Serviceable> {
    fn clone(&self) -> Box<dyn Serviceable> {
        self.clone_box()
    }
}

pub type ServiceBuilder = fn() -> Box<dyn Serviceable>;



