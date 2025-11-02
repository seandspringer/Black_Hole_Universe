# Black Hole Universe

Copyright (c) 2025 Sean Springer

> Note: This README is expected to change as this project evolves

**Black Hole Universe** is a 2D N-body simulator / game built entirely in
`Rust` using the `Bevy` game engine.  
Can you find structure in chaos?

**Black Hole Universe** models a hypothetical _spherical_ universe
filled mostly with [black holes](https://en.wikipedia.org/wiki/Black_hole)!  
The black holes are very massive and as such, have large gravitational  
potential which will attact other objects. If you get swallowed by a black  
hole, you become a part of the black hole!

The simulation will also contain a small amound of planets mixed throughout  
the universe. **The Goal**: by adjusting the initial conditions of the  
universe, can you find a state that produces a long lived orbit for one  
or more of your planets so that life could exist (as we know it)? This concept  
can be generalized to producing structure from a starting chaotic state governed  
only by [Newton's Laws of Gravitation](https://en.wikipedia.org/wiki/Newton%27s_law_of_universal_gravitation),
the system's initial conditions, and the following evolutionary operations:

1. Because the Universe is spherical, objects wrap around the edges of the 2D projection
2. The Collision of a black hole and another black hole creates a single (larger) black hole  
   according to the masses of the colliding black holes
3. The collision of a planet with a black hole "destroys" the planet and increases the size  
   of the consuming black hole according to the mass of the planet
4. The collision of two planets will cause each planet to "fracture in half" effectively  
   creating 4 new planets with mass = 1/2 the parent that it fractured from
5. Planets that fracture below a certain radius effectively become "dust" and all hope  
   of producing life on that "dust" has been vanguished
6. All objects will obey [Newton's Laws of Motion](https://en.wikipedia.org/wiki/Newton%27s_laws_of_motion)
   to some discrete approximation
7. The radius of a black hole is [3 times its mass](https://blackholes.stardate.org/resources/article-structure-of-a-black-hole.html)
8. The radius of planets will follow as [mass raised to 0.27 power](https://www.aanda.org/articles/aa/full_html/2024/06/aa48690-23/aa48690-23.html#F1)  
   as is approximately uniform for planets of mass approximately as large as Earth

Most of the initial conditions will be stochasitic with probability determined by  
[Gaussian statistics](https://en.wikipedia.org/wiki/Gaussian_process). However, in general,
the Gaussian mean and standard deviation  
may be varied by the user to produce unique outcomes. The following is a list of  
variables that may be user-adjustable and may effect the outcome of the simulation:

1. Population of Black Holes
2. Population of Planets
3. Velocity Distributions (Gaussian)
4. Mass Distributions (Gaussian)
5. Initial Locational Density (Gaussian)

## Wish List

The following are some features that may be added if time permits:

- Include Unique "User" Planet with Determined Start and Velocity
- Include Dark Matter
- Black Hole Evaporation
- Big Bang Aproximation Simulation
- Spontaneous White Hole Formations
- Spontaneous Worm Hole Formations

## Build and Run

At least while this project is being actively constructed  
the `Bevy` game engine has been set to be dynamically linked  
rather than statically. This drastically reduced compile times  
allowing for faster updates. To Disable dynamic linking of `Bevy`  
open the `Cargo.toml` file and remove the `dynamic_linking` option  
from the `features` array.

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

## License

This program is licensed under the "MIT License". Please  
see the file `LICENSE` in the source distribution of this  
software for license terms.
