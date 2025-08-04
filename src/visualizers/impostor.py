#!/usr/bin/env python

import time
import struct
import os
from sys import platform
import tempfile

def get_shared_path(filename):
    if platform == "win32":
        return tempfile.gettempdir() + filename
    else:
        return '/dev/shm/' + filename

# Paths to the coffeevis files.
audio_path = get_shared_path('coffeevis_audio.bin')
program_path = get_shared_path('coffeevis_program.txt')
commadn_path = get_shared_path('coffeevis_command.txt')

file_audio = open(audio_path, mode='rb')
file_commands = open(commadn_path, mode='w')
file_program = open(program_path, mode='r')

# We track for modified time and read when it's newer, instead of reading
# the contents every time.
audio_modified_time = 0.0
program_modified_time = 0.0

# This is similar to AudioBuffer in coffeeivs. Audio data only updates
# a few times per second, so we only use a chunk of it, then rotate the
# array to get the next chunk. This also records the number of rotations
# happened and set rotate_size to the average, smoothening the updates.
rotate_times = 0
rotate_size = 300

# Default program info
# Will be updated in the event loop.
program_w = 50
program_h = 50
duration = 1 / 72

def read_to_array(f) -> list[tuple[float, float]]:
    incr = 8

    file_audio.seek(0)
    fileContent = file_audio.read()

    length = len(fileContent)
    sample_array = []

    for i in range(0, length, incr):
        pair = struct.unpack("@ff", fileContent[i:i+incr])
        sample_array.append(pair)

    return sample_array

sample_array = read_to_array(file_audio)

while True:
    mtime_prog = os.path.getmtime(program_path)

    # Checks if program txt file was updated.
    if program_modified_time != mtime_prog:
        program_modified_time = mtime_prog
        file_program.seek(0)
        file_program_contents = file_program.read()

        file_program_contents = file_program_contents.split()

        if len(file_program_contents) != 4:
            break

        program_w = int(file_program_contents[1])
        program_h = int(file_program_contents[2])
        refresh_rate = int(file_program_contents[3])
        duration = 1 / refresh_rate

    mtime_audio = os.path.getmtime(audio_path)

    if audio_modified_time != mtime_audio:
        audio_modified_time = mtime_audio

        sample_array = read_to_array(file_audio)

        # Computes ideal rotate_size
        rotate_size = len(sample_array) // (rotate_times + 1)
        rotate_times = 0
    else:
        rotate_times += 1
        sample_array = sample_array[rotate_size:] + sample_array[:rotate_size]

    length = len(sample_array) // 3

    file_commands.seek(0)

    # Prefer to write everything to file at once
    string_out = ''

    string_out += 'C 00 00 00 00 o f\n'

    for i in range(length):
        xl = int(sample_array[i][0]*program_h / 2 + program_h /2)
        xr = int(sample_array[i][1]*program_h / 2 + program_h / 2)
        r = int(i * 255 // length)
        gb = 255 - r
        string_out += "C FF {:02x} FF {:02x} o p {:04x} {:04x}\n".format(r, gb, i * program_w // length, xl)
        string_out += "C FF {:02x} {:02x} FF o p {:04x} {:04x}\n".format(r, gb, i * program_w // length, xr)

    file_commands.write(string_out)
    file_commands.flush()

    time.sleep(duration)

print("Program file not found, empty, or an error has occured.")
file_audio.close()
file_commands.close()
file_program.close()
