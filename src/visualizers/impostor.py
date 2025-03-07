#!/usr/bin/env python

from collections import deque
from ast import arg
import time
import struct
import os
import math

audio_path = '/dev/shm/coffeevis_audio.bin'
program_path = '/dev/shm/coffeevis_program.txt'
commadn_path = '/dev/shm/coffeevis_command.txt'

file_audio = open(audio_path, mode='rb')
file_commands = open(commadn_path, mode='w')
file_program = open(program_path, mode='r')

audio_modified_time = 0.0
program_modified_time = 0.0

smooth = 0.0
index = 0
rotate_size = 300
rotate_times = 0

program_w = 50
program_h = 50
duration = 1 / 72

def read_to_array(f) -> tuple[list[float], list[float]]:
    incr = 8

    file_audio.seek(0)
    fileContent = file_audio.read()

    length = len(fileContent)
    arrayleft = []
    arrayright = []

    for i in range(0, length, incr):
        pair = struct.unpack("@ff", fileContent[i:i+incr])
        arrayleft.append(pair[0])
        arrayright.append(pair[1])

    return (arrayleft, arrayright)

arrayleft, arrayright = read_to_array(file_audio)


while True:

    mtime_prog = os.path.getmtime(program_path)

    if program_modified_time != mtime_prog:
        program_modified_time = mtime_prog
        file_program.seek(0)
        file_program_contents = file_program.read()

        file_program_contents = file_program_contents.split()

        if len(file_program_contents) != 4:
            break

        program_w = int(file_program_contents[1])
        program_h = int(file_program_contents[2])
        refresh_rate = int(file_program_contents[3]) / 1000
        duration = 1 / refresh_rate

    mtime_audio = os.path.getmtime(audio_path)

    if audio_modified_time != mtime_audio:
        audio_modified_time = mtime_audio
        arrayleft, arrayright = read_to_array(file_audio)
        rotate_size = len(arrayleft) // (rotate_times+1)
        rotate_times = 0
    else:
        rotate_times += 1
        arrayleft = arrayleft[rotate_size:] + arrayleft[:rotate_size]
        arrayright = arrayright[rotate_size:] + arrayright[:rotate_size]

    arrl = arrayleft[:200]
    arrr = arrayright[:200]
    length = len(arrl)

    file_commands.seek(0)

    for i in range(len(arrl)):
        xl = int(arrl[i]*program_h//2 + program_h//2)
        xr = int(arrr[i]*program_h//2 + program_h//2)
        r = int(i * 255 // length)
        gb = 255 - r
        file_commands.write("C FF {:02x} FF {:02x} o p {:04x} {:04x}\n".format(r, gb, i * program_w // length, xl))
        file_commands.write("C FF {:02x} {:02x} FF o p {:04x} {:04x}\n".format(r, gb, i * program_w // length, xr))


    file_commands.flush()

    time.sleep(duration)

print("Program file not found, empty, or an error has occured.")
file_audio.close()
file_commands.close()
file_program.close()
