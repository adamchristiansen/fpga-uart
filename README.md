# FPGA UART

This project implements a serial UART for use in FPGAs. It also includes an
FPGA design that implements a basic echo server and an executable to test it.

This project was designed for and tested on the Digilent Arty A7 (with a Xilinx
Artix-7 XC7A35T), though it should work on other boards with some modifications
to the the constraints and clock.

## Building the HDL

Open Xilinx Vivado and select `Tools > Run Tcl Script...`, then select the
`generate_project.tcl` script in the file exporer. The script will run and
produce the Vivado project in a new `proj/` directory by importing all of the
project sources. If the project fails to be created, it is most likely that the
`proj/` directory already exists.

## Serial Tester

The `serial_tester` program is used to evaluate the performance of a connected
serial device that acts like an echo server. It accepts one parameter, which is
the name of the port to test.

To build it, navigate to the `serial_tester/` directory and run

    $ cargo build

It can be run with `cargo` as well. For example, to run a test on a port named
`/dev/serial/by-id/device` at 115200 baud, testing each packet size 20 times,
with packet sizes of 10, 50, and 100, use

    $ cargo run -- /dev/serial/by-id/device -b115200 -r20 -s10 -s50 -s100

For a list of options that can be used with the testing program, use

    $ cargo run -- -h
