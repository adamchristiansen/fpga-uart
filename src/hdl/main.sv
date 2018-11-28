`timescale 1ns / 1ps

/// This implements a basic echo server where the UART receiver is tied to the
/// transmitter.
///
/// # Ports
///
/// *   [clk] is the 100MHz off-chip clock.
/// *   [rst] is the active high system reset signal.
/// *   [usb_rx] is the receive end of the serial.
/// *   [usb_tx] is the transmit end of the serial.
module main(
    input logic clk,
    input logic rst,
    input logic usb_rx,
    output logic usb_tx);

    // The clock divider to get 115200Hz from 100MHz
    localparam int DIVIDER = 868;

    // The data received and sent over serial
    logic [7:0] d;

    // Indicates that a byte was received
    logic recv_rdy;

    uart_recv #(.DIVIDER(DIVIDER)) uart_recv(
        .clk(clk),
        .rst(rst),
        .rx(usb_rx),
        .d(d),
        .rdy(recv_rdy)
    );

    uart_send #(.DIVIDER(DIVIDER)) uart_send(
        .clk(clk),
        .rst(rst),
        .d(d),
        .send(recv_rdy),
        .rdy(/* Not connected */),
        .tx(usb_tx)
    );

endmodule
