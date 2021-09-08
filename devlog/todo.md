
##todos august	

- i'd like to be able to detect and show hands; so camera module, wired to recognizer; and then spitting out to graphics layer

- i'd like to detect and show walls, floors from slam and maybe bounce a ball off surfaces - maybe at same time

- does it make sense to use tokio or redshirt as my foundation and do messaging between modules using that instead?

- get the parser integrated for my grammar -> maybe switch to v8?

- be able to formally declare and load modules with permissions; the idea is that i should be able to declare modules, declare what messages are permitted between them, and then formally wire them together; the test case being simply a camera and some computer vision and then a display


- maybe write the desktop as a native app

	- i'd also like to be able to produce the desktop ux; which is an input box, that can then go and load a manifest from a remote site and produce the applications specified

	- i'd also like a list of applications somewhere; maybe a side-bar; and maybe a way to choose which one is a focus... i guess each one is going to have its own display or they all have to write to the same bevy scenegraph and then i have to flick various branches off or on

	- i'd like to let modules query the scene as well at a high level; and decorate it with enhancements; as well as some kind of restricted perms


## hands

mediapipe/blazepalm?
	https://ai.googleblog.com/2019/08/on-device-real-time-hand-tracking-with.html
	https://radu-matei.com/blog/tensorflow-inferencing-wasi/

	ok this makes sense, basically tensorflow lite can run in wasm, it gets a trained file, and can spit out hand poses
	aug 6 2021 -> so i think i can make this work

openxr? [ revisit later ]
	https://github.com/Ralith/openxrs
	https://github.com/gfx-rs/gfx/issues/3219
	https://github.com/blaind/xrbevy            <- very interesting
	https://monado.dev/
	https://gitlab.freedesktop.org/monado/monado
	https://www.collabora.com/news-and-blog/blog/2021/06/17/bag-of-freebies-xr-hand-tracking-machine-learning-openxr/
	https://www.collabora.com/news-and-blog/blog/2021/04/20/continuous-3d-hand-pose-tracking-using-machine-learning-and-monado-openxr/
	https://github.com/omigroup/OMI
	https://github.com/GodotVR/godot_openxr
	https://gitlab.freedesktop.org/monado/demos/openxr-simple-example
	https://gitlab.freedesktop.org/monado/demos/openxr-simple-playground

