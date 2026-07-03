# Notes when prototyping
* Require at least two MCUs, two CAN tranceivers, and two 120 Ohm terminating resistors to test CAN connection.
* Baud rate must be the same.
* Both MCUs should have the **same** system clock speed for maximum reliability, otherwise reception is inconsistent.
    * Found from testing a 80 MHz clock with a 40 MHz clock speed at 100 kbps.
    * May not be true, the 40 MHz clock speed might have been to slow, needs more testing.
    * Both system clock speeds were fast enough to set baud rate to 100 kbps
* Link to STM32 CAN baud rate calculator https://www.teachmemicro.com/stm32-can-bus-configuration-calculator/
* Charging UI fps = 6
* Running UI fps = 25
* Standby UI fps = 5
* Screen Clear = 630 ms
