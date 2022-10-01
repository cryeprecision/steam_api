# Library for concurrent requests to the Steam-API

## TODO

- Build a custom `BufferUnordered` that is timed, as in it only
  dispatches a new future to execute after a set amount of time.
  - Just copy-paste-modify from `BufferUnordered` :^)
