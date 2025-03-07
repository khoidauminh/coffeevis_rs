# The Foreign Communicator Module

This is a method for coffeevis to communicate with other
programs via tmpfs files.

This allows writing visualizers in other languages.
See an example in [impostor.py](src/visualizers/milk/impostor.py)

## How it's done

When invoked (via the `--foreign` flag), coffeevis opens
3 files: the audio binary file, the commands text file,
and the program text file.

```
/dev/shm/coffeevis_audio.bin
```
Periodically updates with a continuous audio data array (typically 2000 samples)

```
/dev/shm/coffeevis_commands.txt
```
This is where external programs send in draw commands for coffeevis to render.

```
/dev/shm/coffeevis_program.txt
```
This is where information about the program will be stored.

To get started, launch coffeevis with the `--foreign` flag and run [Impostor.py](../visualizers/impostor.py).

For details, see [foreign.rs](foreign.rs)
