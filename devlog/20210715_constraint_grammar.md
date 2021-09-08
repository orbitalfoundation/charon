# Constraint Grammar

I want a grammar that lets me formally define relationships between units of computation at runtime. By relationships I mean several kinds of ideas:

1. State transport and Interface Definitions. For example myfunction(int a,int b) can throw an error, and is asynchronous, and returns a promise or a float. It might even be nice to specify time limits.

2. Permissions; to specify if one module can call a method of another module basically.

3. Limits; to specify how many resources one module can ask of another.

4. Capabilities; usually based on a key or permission [ https://en.wikipedia.org/wiki/Capability-based_security ]

5. Semantic Intent; for example "place this photo on a wall" as opposed to "place this photo at xyz"


## There are different ways different people talk about this idea:

1. WASM Interface Types. https://radu-matei.com/blog/wasm-api-witx/ . This way of thinking about how modules talk to each other is based around an idea of "capabilities". Given modules can export an API and others can consume that API and an intermediary may set constraints or limits. See also https://www.tutorialspoint.com/cplusplus/cpp_interfaces.htm . Also see https://hacks.mozilla.org/2019/08/webassembly-interface-types/

2. WASI. This is a sloppy idea for formalizing how WASM modules talk to a "system" which is a presupposed magical set of capabilities that exists outside of WASM modules. https://hacks.mozilla.org/2019/03/standardizing-wasi-a-webassembly-system-interface/ . The conceptual defect here is that the "system" really should just conceptually be another module, not a magical thing with special hard coded properties.



I want to be able to load up modules and wire them together with spermissions or capabilities being formally expressed. 

## Other discussions around state transport between WASM modules

https://alexene.dev/2020/08/17/webassembly-without-the-browser-part-1.html
https://www.youtube.com/watch?v=vqBtoPJoQOE
https://docs.wasmtime.dev/examples-rust-hello-world.html
https://docs.wasmtime.dev/examples-rust-wasi.html
https://docs.wasmtime.dev/examples-rust-multi-value.html
https://hacks.mozilla.org/2019/03/standardizing-wasi-a-webassembly-system-interface/
https://labs.imaginea.com/talk-the-nuts-and-bolts-of-webassembly/
https://kevinhoffman.medium.com/introducing-wapc-dc9d8b0c2223
https://github.com/wasmCloud/wascap
https://github.com/wasmCloud
https://www.ralphminderhoud.com/blog/rust-ffi-wrong-way/
https://doc.rust-lang.org/nomicon/ffi.html
https://www.youtube.com/watch?v=B8a01m8B6LU
https://rise.cs.berkeley.edu/projects/erdos/
https://www.w3.org/2018/12/games-workshop/slides/08-web-idl-bindings.pdf
https://docs.microsoft.com/en-us/windows/mixed-reality/mrtk-unity/features/ux-building-blocks/app-bar

