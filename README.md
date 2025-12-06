# Black Hole Universe

Copyright (c) 2025 Sean Springer

**Black Hole Universe** is a 2D N-body simulator / game built entirely in
`Rust` using the `Bevy` game engine.  
Can you find structure in chaos?

**Black Hole Universe** models a hypothetical _spherical_ universe
filled mostly with [black holes](https://en.wikipedia.org/wiki/Black_hole)!  
The black holes are very massive and as such, have large gravitational  
potential which will attact other objects. If you get swallowed by a black  
hole, you become a part of the black hole!

The simulation will also contain a single planet whose starting location, initial heading,  
and velocity will be determined by user input.

**The Goal**: by adjusting the initial conditions of the  
universe, can you find a state that produces a long lived orbit for your planet  
so that life could exist (as we know it)? This concept  
can be generalized to producing structure from a starting chaotic state governed  
only by [Newton's Laws of Gravitation](https://en.wikipedia.org/wiki/Newton%27s_law_of_universal_gravitation),
the system's initial conditions, and the following evolutionary operations:

1. Because the Universe is spherical, objects wrap around the edges of the 2D projection
2. The Collision of a black hole and another black hole creates a single (larger) black hole  
   according to the masses of the colliding black holes
3. The collision of a planet with a black hole "destroys" the planet and increases the size  
   of the consuming black hole according to the mass of the planet
4. Gravity will "wrap around" the frame of the universe to more accurately model a spherical universe
5. All objects will obey [Newton's Laws of Motion](https://en.wikipedia.org/wiki/Newton%27s_laws_of_motion)
   to some discrete approximation
6. The radius of a black hole is [3 times its mass](https://blackholes.stardate.org/resources/article-structure-of-a-black-hole.html)

Most of the initial conditions will be stochasitic with probability determined by  
[Gaussian statistics](https://en.wikipedia.org/wiki/Gaussian_process). However, in general,
the Gaussian mean and standard deviation  
may be varied by the user to produce unique outcomes. The following is a list of  
variables that may be user-adjustable and may effect the outcome of the simulation:

1. Population of Black Holes
2. Mass Distributions (Gaussian)
3. Velocity Distributions (Gaussian)
4. Initial Locational Density (Gaussian)
5. Starting Location of the Planet
6. Initial Heading of the Planet
7. Initial Velocity of the Planet

## How To Run The Simulation

After succesfully building and running the `Rust` program (see _Build and Run_ below),  
users will be presented with a UI containing the universe and various HUD elements.

1. Total Simulation Time and Black Hole Count: in the top left corner of the HUD are text  
   fields labled `Total Time:` and `Black Holes:`. These are counters for the length of time  
   that the simulation has been running and the current number of black holes present in the  
   universe, respectively.
2. Planetary Existence and Planet Count: in the top right corner of the HUD are text  
   fields labled `World Time:` and `Planets:` which track the duration that the planet has  
   remained unharmed during the simulation and a reference to the planet still being alive,  
   respectively.
3. Black Hole Settings: On the left sidebar, there are 4 slider bars that can be adjusted  
   prior to simulation start. These all effect the initial conditions of the simulation.
   - Count: adjust this slide bar to change the number of black holes in the initial configuration.  
     Note: the Black Hole Count Text field will change to reflect the count of black holes. Adjusting  
     this slider to the right will `increase` the number of black holes in the universe.
   - Masses: adjust the mass (size) distribution of black holes in the universe. Black hole sizes  
     are assigned using random normal distribution sampling. This slide bar moves the mean of the  
     gaussian distribution used for this sampling. Adjusting this slider to the right will `increase`  
     the mean mass of black holes in the universe.
   - Velocity: adjust the initial velocity distribution of the black holes in the universe. Black  
     hole velocities are assigned using random normal distribution sampling. This slide bar adjust the  
     width (standard deviation) of the gaussian distribution used for this sampling. Adjusting this slider  
     to the right will `increase` the width of allowed starting velocities in the universe.
   - Density: adjust the initial density of black holes in the universe. The allowed starting locations of  
     the black holes will be adjusted with this slider. Starting locations are assigned using a random  
     normal distribution and this slider adjust the width (standard deviation) of this location gaussian  
     distribution. Adjusting this slider to the right will `increase` the density which decreases the  
     effective radius of spawn.
4. The Universe: The center black rectangle (surrounded by a thin red border) defines the universe (i.e  
   the playing field). The simulation will progress within this space.
5. Starting the Simulation: the simulation will automatically begin once the user has selected the  
   starting location for their planet and assigned it a heading and initial velocity.
   - Left-clicking anywhere inside the Universe (within the red border) will place a `green` planet at that location
   - _While holding the click_, drag the mouse in the direction you would like the initial heading of the planet to be.  
     The planets direction is dictated by the imaginary line from the starting location to the mouse's current position.
   - The distance between the current mouse position and starting position dictates the planet's initial velocity.  
     The further the mouse is dragged away from the starting position, the faster its initial velocity will be.
   - Once at the desired heading and velocity, release the left mouse btn to start the simulation!
6. Simulation End: The simluation will progress until only one object remains, the Super Massive black hole!
   - At this point, the simulation will stop (all counters and motion will cease).
   - In the bottom right corner of the HUD, a `Restart?` button will appear
   - Clicking this button will reset the simulation, reasigning the universe to the black hole settings current selected.  
     The `Restart?` button will dissapear and you can now readjust the starting configuration (see step `3`).
   - When ready, repeat step `5` to begin the new simulation!

## Runtime Troubleshooting

There are really only 2 potential runtime related issues that have been observed after using the steps provided in  
`How To Run The Simulation`.

1. While setting the initial velocity of the planet via the drag mechanism (see section `5` of `How To Run The Simulation`)  
   the cursor leaves the boundary of the universe. In this case, the planet will be placed (green circle) however the simulation  
   will fail to begin. In this scenario, simply try again! Be sure to release the mouse button while inside the universe.
2. No velocity was chosen for the starting universe. This can occur if the click to place the planet was not held while dragging  
   to set the initial velocity. In this scenario, the planet will be placed but the simulation will fail to begin. Again, simply  
   try again! Be sure to provide at least a small amount of drag prior to releasing the mouse button.

## Build and Run

At least while this project is being actively constructed  
the `Bevy` game engine has been set to be dynamically linked  
rather than statically. This drastically reduced compile times  
allowing for faster updates. To Disable dynamic linking of `Bevy`  
open the `Cargo.toml` file and remove the `dynamic_linking` option  
from the `features` array. Please be paitent as build times (especially the initial)  
can be brutal considering the quantity of dependencies (402 by latest count).

Debug and release builds follow the standard `Cargo` command syntax:

```bash
cargo run
cargo run --release
```

the Debug build is recommended for developers while users of the  
simulator can invoke the `--release` argument for improved performance.

Further runtime performance updates may be added to the `Cargo.toml` in  
future builds.

## Sources

1. [Getting Started with Bevy](https://bevy.org/learn/quick-start/introduction/)
2. [Unofficial Bevy Cheat Book](https://bevy-cheatbook.github.io/introduction.html)
3. [Performance Enhancing Algorithms](https://patterns.eecs.berkeley.edu/?page_id=193)
4. [Newton's Laws of Gravitation](https://en.wikipedia.org/wiki/Newton%27s_law_of_universal_gravitation)
5. [Newton's Laws of Motion](https://en.wikipedia.org/wiki/Newton%27s_laws_of_motion)
6. [Newton's Equations of Motion](https://en.wikipedia.org/wiki/Equations_of_motion)
7. [Black Hole Mass to Radius Relationship](https://blackholes.stardate.org/resources/article-structure-of-a-black-hole.html)
8. [Planetary Mass to Radius Relationship](https://www.aanda.org/articles/aa/full_html/2024/06/aa48690-23/aa48690-23.html#F1)
9. [Line Intersection Algebra](https://www.topcoder.com/thrive/articles/Geometry%20Concepts%20part%202:%20%20Line%20Intersection%20and%20its%20Applications)
10. [Cartersian and Standard Form](https://www.sunshine2k.de/articles/algorithm/line2d/linerep2d.html)
11. [Distance Between Point and Line](https://www.splashlearn.com/math-vocabulary/distance-of-a-point-from-a-line#:~:text=The%20shortest%20distance%20between%20point%20and%20line,drawn%20from%20the%20point%20to%20the%20line.)
12. [Distance Between Point and Line 2](https://en.wikipedia.org/wiki/Distance_from_a_point_to_a_line)
13. [Bevy Buttons](https://bevy.org/examples/ui-user-interface/button/)
14. Claude Sonnet 4.5 (free version)

## Future Implementation List

The following are some features that could be added in future releases.  
Most of these were initially planned for the release but were scrapped due to  
time constaints:

- Include More Planets
- Include Dark Matter
- Black Hole Evaporation
- Big Bang Aproximation Simulation
- Spontaneous White Hole Formations
- Spontaneous Worm Hole Formations

## License

This program is licensed under the "MIT License". Please  
see the file `LICENSE` in the source distribution of this  
software for license terms.
