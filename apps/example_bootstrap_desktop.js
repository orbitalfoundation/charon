let w = 600
let h = 600

var system = {

	// this module is slightly special, it helps other modules communicate
	broker: {
	},

	// a display module; it supports opening and painting some pixels to the users face somehow
	display: {
		kind:"display",
		width:w,
		height:h,
	},

	// a desktop; this paints a url input box in a privileged area that other apps cannot obscure
	desktop_app: {
		children:[
			banner: {
				kind:"display::box",
				width:w,
				height:h/10,
				border:"3px solid red",
				children: [
					text: {
						kind:"display::text",
						text:"please enter a url here"
					},
					button: {
						kind:"display::button",
						text:"go",
					}
				]
			},
			userland:{
				kind:"display::box",
				children: [],
			}
		],
		on_touch:function(ev) {
			// do some actual code here if you want
		}
	},

	// an example app
	// it has some view pieces that know how to try get themselves rendered
	// it has a camera that also dynamically manufactures a viewable bit of art
	// it has a segmenter that knows how to paint stuff as well
	/// overall it just captures camera frames and shows faces; it runs a couple of singletons
	friendfinder_app:{
		children:[
			camera: {
				kind:"webcam::singleton"
				// TODO maybe there should be a separate helper to move pixels to view
				// on_tick: ?
			},
			segmenter: {
				kind:"segmenter::singleton"
				// TODO maybe a separate helper will do paint this
			},
			view:{
				kind:"display::box",
				width:w/4,
				height:h/10,
				children: [
				]
			},
			wire1:{
				route:"camera -> segmenter",
			},
		],
	}
}

