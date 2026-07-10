Troubleshooting CAN:
* (CAN Transceiver Datasheet)[https://ww1.microchip.com/downloads/aemDocuments/documents/APID/ProductDocuments/DataSheets/MCP2542FD-MCP2542WFD-4WFD-Data-Sheet-DS20005514C.pdf]
* VDD must be between 4.5-5V, otherwise the transceiver cannot enter Normal Mode
* VIO must be between 1.7-5.5V
* Requires at least one termination resistor.
* Resistance between CANH and CANL should not be less than 60 Ohms
* Supplying 5V to the 12V line will lower the PCBs 5V voltage line to 3.8V, too low for the transceiver to work.
* Need to supply 5V to the 5V header pin for the LCD display to get 5V power.
