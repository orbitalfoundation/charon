
let system = {									// a system is typically rooted with one global in ordinary javascript

	sun: {										// this is a child object with some simple properties
		kind:"3d/light",
		xyz:{x:0,y:0,z:0},
		color:0xffff00,
	},
	earth: {
		kind:"3d/mesh",
		effect1:{								// a child object behavior acting on the parent scope
			kind:"3d/behavior/rotate"
			speed:1,
		},
		effect2: {
			kind:"3d/behavior/collide"
		}
	},
	asteriod: {
		kind:"3d/mesh",
		effect1: {
			kind:"3d/behavior/collide"
		}
	},
}

