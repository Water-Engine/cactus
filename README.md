# cactus
A UCI compatible Chess Engine, inspired by [Sebastian Lague](https://www.youtube.com/c/SebastianLague).

The `engine` folder takes heavy inspiration from Sebastian Lague's videos and [github repository](https://github.com/SebLague/Chess-Coding-Adventure). The algorithms used are almost all discussed in his two-part video series and are also heavily documented online.

At a first glance, you might be confused why there are two different implementations of the core game. This is because, to put it as simply as possible, I wanted to have the engine and gui as decoupled as possible. This allows you to run the gui alone, the gui with the builtin engine, the gui with an external engine, or the engine alone without a gui. This project's design is entirely based upon the idea that the engine has no understanding of the current state in the gui. If the engine wants to make a move, it needs to use `integration.rs` in the `coupling` folder. This folder handles external engine launching as well as allowing engines to interact with the gui. 

If this design choice seems odd or inefficient, _it is_. I am going to stick by this choice however, as the non-engine side of the game does not need the complicated bitboards or algorithms looming over it when brute force methods are acceptable. The engine does not care how slow or fast the gui is, it just runs.
