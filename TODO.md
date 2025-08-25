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
- Update lock files.

## To-Do

- Finish manifest error messages.
