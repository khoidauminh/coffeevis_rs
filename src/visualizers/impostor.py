#!/usr/bin/env python

from collections import deque
from ast import arg
import time
import struct
import os
import math

incr = 8

file_audio = open('/tmp/coffeevis_audio.bin', mode='rb')
file_commands = open('/tmp/coffeevis_command.txt', mode='w')
file_program = open('/tmp/coffeevis_program.txt', mode='r')
duration = 1 / 72

def read_to_array(f) -> tuple[list[float], list[float]]:
    f.seek(0)
    fileContent = f.read()

    length = len(fileContent)
    arrayleft = []
    arrayright = []

    for i in range(0, length, incr):
        pair = struct.unpack("@ff", fileContent[i:i+incr])
        arrayleft.append(pair[0])
        arrayright.append(pair[1])

    return (arrayleft, arrayright)

try:
    smooth = 0.0
    index = 0

    arrayleft, arrayright = read_to_array(file_audio)

    while True:

        file_program.seek(0)
        file_program_contents = file_program.read()

        if len(file_program_contents) == 0:
            break

        file_program_contents = file_program_contents.split()
        program_w = int(file_program_contents[1])
        program_h = int(file_program_contents[2])

        if index > 3:
            arrayleft, arrayright = read_to_array(file_audio)
            index = 0
        else:
            index += 1
            arrayleft = arrayleft[100:] + arrayleft[:100]

        # res = math.fsum(arrayleft) / len(arrayleft)

        arr = arrayleft[:200]

        file_commands.seek(0)

        for i, sample in enumerate(arr):
            sample = arr[i]
            x = int(sample*program_h//2 + program_h//2)
            string = "COMMAND FF FF FF FF over plot {:04x} {:04x}\n".format(i * program_w // len(arr), x)
            file_commands.write(string)

        file_commands.flush()

        time.sleep(duration)
finally:
    print("Program file not found, empty, or an error has occured.")
    file_audio.close()
    file_commands.close()
