
#[link(wasm_import_module = "imports")]
extern "C" {
  fn orbital_dowork(a:i32, b:i32);
}

#[no_mangle]
extern "C" fn run() {
  unsafe {
    orbital_dowork(1,2);
  }
}
