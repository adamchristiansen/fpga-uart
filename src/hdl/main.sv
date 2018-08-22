`timescale 1ns / 1ps

/// This implements a basic echo server where the UART receiver is tied to the
/// transmitter.
///
/// # Ports
///
/// *   [clk] is the 100MHz off-chip clock.
/// *   [reset] is the active high system reset signal.
/// *   [usb_rx] is the receive end of the serial.
/// *   [usb_tx] is the transmit end of the serial.
module main(
    input logic clk,
    input logic reset,
    input logic usb_rx,
    output logic usb_tx);

    // The clock divider to get 115200Hz from 100MHz
    localparam int DIVIDER = 868;

    // The data received and sent over serial
    logic [7:0] data;

    // Indicates that a byte was received
    logic receive_ready;

    uart_receive #(.DIVIDER(DIVIDER)) uart_receive(
        .clk(clk),
        .reset(reset),
        .rx(usb_rx),
        .data(data),
        .ready(receive_ready)
    );

    uart_transmit #(.DIVIDER(DIVIDER)) uart_transmit(
        .clk(clk),
        .reset(reset),
        .data(data),
        .send(receive_ready),
        .ready(/* Not connected */),
        .tx(usb_tx)
    );

endmodule
