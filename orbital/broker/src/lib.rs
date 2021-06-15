
use crossbeam::channel::*;

use service::*;

///
/// A datastructure to internally register services
///

pub struct ServiceWrapper {
    pub sid: SID,
    pub name: String,
    pub send: Sender<Message>,
    pub subscriptions: std::cell::RefCell<std::collections::HashSet<String>>,
}

///
/// A broker service that plays something of a special role in that it helps other services talk to each other
///

#[derive(Clone)]
pub struct Broker {
}
impl Broker {
    pub fn new() -> Box<dyn Serviceable> {
        Box::new(Self{})
    }
}
impl Serviceable for Broker {
    fn name(&self) -> &str { "broker" }
    fn stop(&self) {}
    fn start(&self, _name: String, _sid: SID, send: Sender<Message>, recv: Receiver<Message> ) {        
        let name = self.name();
        let _send = send.clone();
        let recv = recv.clone();
        println!("broker starting {}",_sid);

        let _thread = std::thread::Builder::new().name(name.to_string()).spawn(move || {
            let mut registry = std::collections::HashMap::<SID,ServiceWrapper>::new();
            while let Ok(message) = recv.recv() {
                match message {

                    Message::Subscribe(sid,topic) => {
                        if !registry.contains_key(&sid) {
                            println!("Broker: forcing entry for non-existent app {} ('{}') to topic '{}'",sid,registry[&sid].name,topic);
                            let (_trashsend,_trashreceive) = unbounded::<Message>();
                            let wrapper = ServiceWrapper {
                                sid: sid,
                                name: "no name yet".to_string(),
                                send: _trashsend,
                                subscriptions: std::cell::RefCell::new(std::collections::HashSet::new()),
                            };
                            registry.insert(sid,wrapper);
                        }
                        println!("Broker: subscribing app {} ('{}') to topic '{}'",sid,registry[&sid].name,topic);
                        registry[&sid].subscriptions.borrow_mut().insert(topic);
                    },

                    Message::Unsubscribe(sid,topic) => {
                        println!("Broker: unsubscribing app {} ('{}') from topic '{}'",sid,registry[&sid].name,topic);
                        registry[&sid].subscriptions.borrow_mut().remove(&topic);
                    },

                    // hack, forward share objects...
                    Message::Share(sharedmemory) => {
                        // repost event objects 
                        for target in &registry {
                            if target.1.subscriptions.borrow_mut().contains(&"/display".to_string()) {
                                //let mut ptr = sharedmemory.lock().unwrap();
                                //let mut sharedmemory = Arc::new(Mutex::new(Box::new(ptr)));
                                let _res = target.1.send.send(Message::Share(sharedmemory));
                                break;
                            }
                        }
                    },

                    Message::Event(topic,data) => {
                        // repost event objects 
                        for target in &registry {
                            if target.1.subscriptions.borrow_mut().contains(&topic) {
                                let _res = target.1.send.send(Message::Event(topic.clone(),data.clone()));
                            }
                        }
                    },

/*
                    // this is disabled because of a design quirk in the way that winit demands the main thread
                    // so we have to start all services on the main thread sadly
                    // instead the idea of a Message::Channel is used to tell the broker about how to talk to a service

                    Message::Add(service) => {

                        let sid: SID = rand::random::<SID>();
                        let (localsend,localrecv) = unbounded::<Message>();
                        let instance = service();
                        let name = instance.name().to_string();

                        let wrapper = ServiceWrapper {
                            sid: sid,
                            name: name.clone(),
                            send: localsend,
                            subscriptions: std::cell::RefCell::new(std::collections::HashSet::new()),
                        };
                        registry.insert(sid,wrapper);

                        instance.start(name,sid,send.clone(),localrecv);

                    },
*/
                    Message::Channel(sid,name,channel) =>{
                        if !registry.contains_key(&sid) {
                            println!("Broker: added channel for {} {}",sid,name);
                            let wrapper = ServiceWrapper {
                                sid: sid,
                                name: name,
                                send: channel,
                                subscriptions: std::cell::RefCell::new(std::collections::HashSet::new()),
                            };
                            registry.insert(sid,wrapper);
                        } else {
                            println!("Broker: revising existing channel for {} {}",sid,name);
                            registry.remove(&sid);
                            let subscriptions = registry[&sid].subscriptions.clone();
                            let wrapper = ServiceWrapper {
                                sid: sid,
                                name: name.clone(),
                                send: channel,
                                subscriptions: subscriptions,
                            };
                            registry.insert(sid,wrapper);
                        }
                    },

/*
                    // this was a quick a test to boot a wasm blob
                    // i'd prefer not to hard code this directly into the broker however

                    Message::BrokerGoto(url) => {

                        if url.len() < 1 {
                            return
                        }

                        let mut found = false;

                        for (_key, value) in &registry {
                            if value.name.eq(&url) {
                                println!("found {} {}", value.name, url );
                                found = true
                            }
                        }

                        if found == true {
                            return
                        }

                        let sid: SID = rand::random::<SID>();
                        let (localsend,localrecv) = unbounded::<Message>();
                        let instance = Wasm::new();
                        let name = url;

                        println!("Broker: added channel for {} {}",sid,name);

                        let wrapper = ServiceWrapper {
                            sid: sid,
                            name: name.clone(),
                            send: localsend,
                            subscriptions: std::cell::RefCell::new(std::collections::HashSet::new()),
                        };
                        registry.insert(sid,wrapper);
                        instance.start(name,sid,send.clone(),localrecv);
                    },
*/

                    //_ => {}
                }
            }
        });
    }
}




///
/// A helpful bootstrapper that kicks off a few services and wires them up to the broker
///

pub fn bootstrap(services: &[ServiceBuilder] ) {

    // specially build channels for broker to send and receive messages
    let (brokersend,brokerrecv) = unbounded::<Message>();

    // specially start broker with public channels (broker MUST be the first service)
    let _ = services[0]().start("broker".to_string(),0,brokersend.clone(),brokerrecv.clone());

    // start remaining services, passing them a way to talk to the broker, and telling broker about them
    for i in 1..services.len() {
        let sid: SID = rand::random::<SID>();
        let (localsend,localrecv) = unbounded::<Message>();
        let instance = services[i]();
        let name = instance.name().to_string();
        let _ = brokersend.send(Message::Channel(sid,name.clone(),localsend));
        instance.start(name,sid,brokersend.clone(),localrecv);
    }
}




