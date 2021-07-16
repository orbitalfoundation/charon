
use crossbeam::channel::*;
use std::error::Error;
use std::fmt;

use service::*;

//////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct MyError(String);

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "There is an error: {}", self.0)
    }
}

impl Error for MyError {}

//////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct Wasm {}
impl Wasm {
    pub fn new() -> Box<dyn Serviceable> {
        Box::new(Self{})
    }
}
impl Serviceable for Wasm {
    fn name(&self) -> &str { "Wasm" }
    fn stop(&self) {}
    fn start(&self, name: String, _sid: SID, send: Sender<Message>, recv: Receiver<Message> ) {
        let _thread = std::thread::Builder::new().name(name.to_string()).spawn(move || {
           let _ = wasm2(name,send,recv);
        });
    }
}



use wasmtime::*;
use anyhow::Result;
//use wasmtime_wasi::{sync::WasiCtxBuilder, Wasi};


fn wasm2(name: String, _send: Sender<Message>,_recv:Receiver<Message>) -> Result<(), Box<dyn Error>> {

    let send2 = _send.clone();

    // start engine once
    println!("Initializing...");
    let engine = Engine::default();
    let store = Store::new(&engine);

    // compile code once
    println!("Compiling module...");
    let module = Module::from_file(&engine,name)?;

    // attach callbacks

    let orbital_dowork_func = move |_a:i32, _b:i32 | {
        println!("wasm::orbital::dowork called");
        //let _ = send2.send(Message::Event("/camera".to_string(),"WASM->Camera: Give me a Frame".to_string()));
        //let _ = send2.send(Message::Event("/display".to_string(),"WASM->Display: Show Frame".to_string()));
        let _ = send2.send(Message::Event("/display".to_string(),"manycubes".to_string()));
    };

    let orbital_dowork_func = Func::wrap(&store,orbital_dowork_func);

    /*
    let drawcube_type = FuncType::new([ValType::I32, ValType::I32].iter().cloned(),[].iter().cloned());
    let drawcube_func = |args1: i32, args2 :i32 , _results :i32| {
        println!("Calling back...");
        println!("... {} {}", args[0].unwrap_i32(), args[1].unwrap_i32());
        //let _ = send2.send(Message::Event("/display".to_string(),"cubes".to_string()));
        Ok(())
    };
    */
    //let add = Func::wrap(&store, |a: i32, b: i32| -> i32 { a + b });
    //let double = Func::new(&store, double_type, |_, params, results| {
    //let drawcube_func = Func::new(&store, drawcube_type, drawcube_func);

    // Instantiate.
    println!("Instantiating module...");
    let instance = Instance::new(&store, &module, &[orbital_dowork_func.into()])?;

    // Extract exports.
    println!("Extracting export...");
    let run = instance.get_typed_func::<(), ()>("run")?;

    // Call `$g`.
    println!("Calling run");
    run.call(())?;

    println!("Printing result...");


    Ok(())
}

