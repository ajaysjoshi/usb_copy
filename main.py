#! /usr/bin/python3

import ctypes


rust_lib = ctypes.CDLL("./libs/libusbcopy.so")

returned = rust_lib.rusb_list()
print("Got back ", ctypes.cast(returned,ctypes.c_char_p).value.decode())

