# Cedar

A minimal C build system with basic incremental compilation, 
quick project creation, and no external dependencies.

## Installation

> cargo install --git "<https://github.com/juniensis/cedar.git>"

## Usage

  cedar \[COMMAND\] \[OPTIONS\]

  Commands:

  - new ->      Creates a new project under the given name.
  - init ->     Creates a new project in the current working directory.
  - build ->    Compiles the project.
  - clean ->    Deletes all temporary build files.
  - run ->      Compiles then runs the project.

  Options:
  - --git      Initializes the project as a git repository when created.
