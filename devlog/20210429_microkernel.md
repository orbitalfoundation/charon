
# Orbital Browser Kernel - April 29 2021

## Kernel first test

At this time I'm first focusing on a microkernel architecture. The goal is to support dynamically loaded wasm modules and a messaging bus and shared memory.

In this test there is one internal module/app that exercises a microkernel with the following use case: a webcam captures a video feed, it is fed to a face segmenter, and results are painted to the display.

Note this is all super early, the code is rough, very fragile. It's not usable for any kind of real world purpose yet. It's purely a design-in-code sketch at the moment. The plan is to keep iterating on this core however.

## How to run this version

Here is how to run the r4 demo (this may be obsolete later)

git checkout https://github.com/orbitalweb/rev1
cd rev1
cargo run --release

This currently runs ONLY on a macbook pro - and it needs to be a fast machine.

Here is what you should be able to see and do in this version:

	- the current test brings up a window
	- a webcam is started automatically
	- face segmentation is started automatically
	- you can type in an URL in the menu bar at the top of the screen
		- you may have to hit delete your text is backwards (this is a bug)
		- you have to hit the go button after you type in your text
		- cubes.wasm -> load some cubes
		- friendfinder.wat -> face finder -> turned on always for now

## Enhancements noted for next versions

	- each app should hash or identify a handle on what it owns so display can show it or not as it wishes
	- paint a list of apps both live and stashed on a side bar
	- broker or somebody needs to be able to report a list of existing apps
	- broker should actually persist apps as well; not just pretend to
	- display needs to be able to paint a list of running apps
	- running apps should be clickable and there should be state on them; permissions and so on
	- also need to be able to switch between apps; give certain apps focus

## General design thoughts around this version: Kernel Architecture approach

The approach I took was to load and run wasm blobs as driven by a multithreaded kernel written in Rust. Conceptually then this can be considered similar to a Desktop or Steam or any kind of modern app manager. It loads apps (wasm blobs), runs them (pre-emptive multithreading), allows messaging between them (crossbeam). It's focused around an idea of downloading persistent or durable applications that it then helps the user manage. Eventually blobs will be able to have a list of dependencies on other blobs; not dissimilar from ordinary package managers that we're used to such as NPM, Crates or WAPM. In this framing there's an idea of "units of computation" that can be dynamically fetched over the wire and that can run in a "computational soup"; or effectively a microservices / microkernel architecture. These computational units can respond to events that can be local (user input) or listen to other traffic as well. In this respect it's aspirationally similar to Fastly Lucet.

These are the pieces I built out for this version (subject to change):

1. Service or module. A service is a wasm blob that defines a unit of computation. I use the term "blob" "wasm blob" "module" or "unit of computation" interchangeably. The goal of this product will be to dynamically ship behavior, not just static layouts or a DSL. There are two flavors of execution:

	Rust Services : There are built-in or hard-coded services which implement raw/unsafe access to devices (camera inputs, display outputs).

	WASM Services : WASM services load up a WASM blob from any remote source, on the fly, at runtime. These are "user apps".

2. Threads. Services are separate pre-emptive threads. It's important to us to have pre-emptive threading. WASM services can use this as well.

3. Messages. Services may message each other through crossbeam channels across threads.

4. Broker. Right now there is a special discovery service that brokers messages. It implements only pub sub for now (no shared memory messages yet).

### Larger insights:

There's some insight we've had already, and here are some of the areas for improvement ( note that this entire code base will likely be thrown away but some of the patterns will remain):

1. Messaging is fairly simple right now. Later services should be able to directly bind to each other, including having shared memory.

2. Standards; right now all device access (camera inputs, displays) are all completely custom; we'd want to standardize on conventions (WebXR?).

3. Graphics; we want to dramatically improve graphics rendering and output to have a vastly more capable visualization experience for users.

4. UX; we want to improve the built in UX to include a desktop, an enumeration of all apps and services and a management panel overall.

5. Scene Graph? I need some kind of inter module shared display retained mode abstraction / scenegraph? Or some kind of hashed list of what is painted to a view? The thing is that modules need to write to shared state; such as reporting where walls and floors are. And then it is also much easier to express high level concepts that are durable in the scene rather than merely sharing display lists.

