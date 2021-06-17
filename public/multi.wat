(module
  (func $drawcube (import "" "drawcube") (param i32 i32) (result i32 i32))

  (func $run (export "run") (param i32 i32) (result i32 i32)
    (local.set 0 (i32.const -1))
    (local.set 1 (i32.const 14))
    (call $drawcube (local.get 0) (local.get 1))
  )

)