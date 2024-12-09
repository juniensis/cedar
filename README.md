# Cedar
### A stripped-down Cargo-like C project manager.
Cedar can create, compile, and run C projects. 

### Usage
Cedar currently has 4 commands: new, init, build, and run. The first two creates new projects either 
with the given name/path, or in the current working directory respectively. Then build compiles the 
program, and run compiles then executes the program.

Upon initializing a new Cedar project, there will be 3 directories generated, src, build, and include, 
as well as a cedar.toml manifest file. All C code in src or include will be linked together when the 
project is ran.


