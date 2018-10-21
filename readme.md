This is an abandoned half-baked experimental project that I might get
back to. Based on file timestamps, I last worked on it around
mid-June 2018.  Putting it up on github mostly for backup purposes, I
might get back to work on this at some point.  It could also be
helpful to someone looking for a simple example of using rust to
process webcam data.

The goal here was to use a webcam to detect whether I was walking,
standing, or sitting. The current heuristic approach was written for
having a dark background (face illuminated by the screens), which made
the problem much easier. It could indeed quickly locate approximately
the top of the head.

The purpose of collecting this information was to accurately track
distance / time walked, remind me to walk, and also to remind me to
switch postural mode.

# Dependencies

* v4l2 - `sudo apt install libv4l-dev`
