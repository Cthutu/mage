# Matt's ASCII Game Engine (MAGE)

This is an engine I wrote to help implement ASCII-based games.  This is the core engine, which only implements a bare minimum to get ASCII text on a window fast using the GPU.

Other modules will provide other systems, like ECS, pathfinding, audio etc.

# Using MAGE

It provides a trait that needs implementing (`Game`) and a builder that allows
you to configure it (such as which font to use etc).  The engine will call the
methods on the trait implementation passing state and input to the game code.

