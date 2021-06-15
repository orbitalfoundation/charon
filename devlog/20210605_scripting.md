# An orbital grammar

The goal is to define a largely declarative grammar that makes it possible for novices to drive orbital; to set up modules, wire them together, handle events.

## Example Script

For example this is a typical document that produces a camera face segmentation app

	let myapp = {
		camera:{
			kind:"modules/display/camera",
		},
		segmenter:{
			kind:"modules/face_segmenter",
		},
		display:{
			kind:"modules/display"
		},
		wire1:"camera -> segmenter",
		wire2:"camera -> display",
		wire3:"segmenter -> display",
		sponsor:"@dorothy",
		copyright:"MIT license",
		on_event:function(ev) { console.log("lifecycle event called"); console.log(ev) }
	}


## A few design choices

General lightweight vanilla json -> basically just use a text declarative-foremost json-like grammar where perhaps the "edges" can be procedural, but the bulk is just declarations.

Crumpling -> I did decide to still support some convenience concepts such as "crumpling"; which is an idea that the grammar in text doesn't have to fully qualify some relationships such as parent { children[ child {} ] } but can just do parent { child {} } if the user wishes.

Manufacturing by message -> As nodes are parsed I effectively turn them into messages, directed at any appropriate handler a section of a graph; so something like root { mydisplay { kind:"display", mylight { kind:"light" } } } would be instancing a node of kind display, and then passing the light to the display (to make if it wants). Each leaf handles the manufacturing of things below itself at will and returns some kind of UUID (or one is shovelled down with the node). Note that it is so much easier to work in JS that I actually write this piece of parser logic in JS itself.

*NO ECS* -> There is no formal ECS model - objects can have children objects that implement a behavior but there is no separate "component" list per object.

*Events* -> There are lifecycle events fired off to "on_event" that can be helpful (procedural logic is allowed on object declarations).

## Special props on nodes

1. "id or uuid" -> This field is reserved internally

2. "name" -> All objects _must_ be named, but the name can be defined in the named hash entry if desired (rather than in the name property).

3. "copy" -> This specifically mixes in any fields of the previous objects specified

4. "kind" -> this specifically binds them to internal system capabilities code or logic if defined

5. "children" -> this specifically is recognized and used by a filesystem like query capability

6. "on_event" -> procedural logic can be attached as to objects that is triggered on certain transitions

7. "on_change" -> what do we want to do with prop events should we pipe them to the object?

## Ongoing: June 12 2021

### engines to try

https://github.com/HiRoFa/spidermonkey_runtime
v8 Deno Rusty
quickjs <- I am using this

### Simplify/Revise Manufacturing ?

	In github.com/anselm/blox we had a pattern of named entities such as this: { mesh: {} }
	This allows only one of a thing in a parent
	Instead do something like { named001_mesh {} or named001 { kind: "mesh" }

### Dealing with callbacks?

	- there are some hassles around callbacks and messaging and so on with rust
		so i think the quickjs call back basically needs to use messages to comunicate with any state or data
		so those messages talk to something else that can deal with tne node creation events; routing as needed

	- i'd love to write more code in quickjs, notably promises and so on, and really drive the system
		it is not perfectly clear how to do that

### Allowing includes ???

### How does servo expose apis to javascript?

In servo we see a pattern where there's a javascript language and then also a pile of web apis.
Is there a formal way that those apis are registered?

One api is document.* such as document.createElement() and document.query() and document.body.addListener()
I think I want to exactly emulate this dom building pattern for my own dom in fact.

Also, browsers in general expose a webglcontext in a similar way; as an object from a canvas context.
And presumably other apis such as https://github.com/servo/webxr are exposed to js in a similar way.

### Webassembly and access to web apis?

How about webassembly itself? Can it see any apis? (no)

	in servo and firefox basically webassembly gets access to webapis in general
		https://developer.mozilla.org/en-US/docs/Web/API
		https://developer.mozilla.org/en-US/docs/WebAssembly/Concepts

"By itself, WebAssembly cannot currently directly access the DOM; it can only call JavaScript, passing in integer and floating point primitive data types. Thus, to access any Web API, WebAssembly needs to call out to JavaScript, which then makes the Web API call. Emscripten therefore creates the HTML and JavaScript glue code needed to achieve this."

### Orbital scripting

I have the option to register callbacks to rust or I can build entire interfaces in javascript. I may as well just stay in js for early tests.

I can also emulate a web component registry
	https://blog.logrocket.com/what-happened-to-web-components/

It's worth noting that the html5 spec is grossly stretched by WHATWG
	https://en.wikipedia.org/wiki/HTML5
