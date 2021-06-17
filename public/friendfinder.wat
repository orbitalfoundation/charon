(module
  (type $t0 (func (param i32 i32)))
  (type $t1 (func))
  (import "imports" "drawcube" (func $_ZN5cubes8drawcube17h0461c18d6abb8fe7E (type $t0)))
  (func $run (export "run") (type $t1)
    (local $l0 i32) (local $l1 i32)
    (local.set $l0
      (i32.const -1))
    (local.set $l1
      (i32.const 14))
    (call $_ZN5cubes8drawcube17h0461c18d6abb8fe7E
      (local.get $l0)
      (local.get $l1))
    (return))
  (table $T0 1 1 funcref)
  (memory $memory (export "memory") 16)
  (global $g0 (mut i32) (i32.const 1048576))
  (global $__heap_base (export "__heap_base") i32 (i32.const 1048576))
  (global $__data_end (export "__data_end") i32 (i32.const 1048576)))
