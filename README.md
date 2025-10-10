# tram
Tram is a small, dynamically-typed, and tree-walked toy-language
resembling a mix of Lua and Python.

The madlibs example in `sample/madlibs.tr` covers most language features.

## history
This language was originally designed and implemented in a week. Then, I
recently (oct 2025) spent some time making it more complete.

Overall, I'm very happy with where it was able to get in such
a short amount of time.

## future work
- a bytecode VM of some sort would be a massive performance improvement. the current
  tree-walking implementation is likely leaving a lot of performance on the table
- implementation of a strong typing system (interested in making a zig-like comptime
  typing system in an interpreted language)
- standard library
- better glue between rust functions and tram language functions. maybe a proc macro?

## features
- dynamic typing
- small standard library with math functions and constants
- if expressions, not statements
