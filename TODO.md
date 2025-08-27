# Agenda

## Structure

Cedar needs to accomplish the following things on the initial run:

- Initialize and read the manifest.
- Find, hash, and take the last modified date of all source files.
- Create build directory.
- Compile all files that need to be compiled and generate -d files.
- Link all .o files.
- Graph dependents of all header files.
- Write all files hash, modify date, and dependents to the lock file.

On subsequent runs, the following must be done:

- Check for new files.
- Check for changes in last modified date.
- If a files last modified date has changed, hash contents to confirm.
- Recompile changed source files and dependents of changed headers.
- Update lock file.

States:

1. No build directory.
2. Build directory but no lock file.
3. Build directory and lock file.

State 1:

1. Create build directory.
2. Move to state 2.

State 2:

1. Create empty lock file.
2. Move to state 3.

State 3:

1. Iterate through source files.
2. If a source file is not in the lock file, add to the compile list.
3. If a file is in the lock file, compare last modified time / hash.
4. If the file is a source file, and it is not up to date, compile it.
5. If the file is a changed header, add its dependents to the compile list.
6. Compile all items on the compile list and update the lock file.
7. Link all .o files and output the binary.
8. Write the lock file.

Structures:

Lock file manager -> Handle lock file reads and writes, allow new data to
be inserted, implements a contains/is changed method.

Build graph -> Read dependency files, walk source directory, generate the
compile list.

Builder -> Orchestrate the lock file and build graph, read the manifest,
compile and link.

Delegation:

State 1 -> Builder
State 2 -> Lock file manager
State 3.1 - 3.5 -> Build graph
State 3.6 - 3.8 -> Builder

Hierarchy:

Builder -> Build Graph -> Lock File Manager

## To-Do

- Finish manifest error messages.
