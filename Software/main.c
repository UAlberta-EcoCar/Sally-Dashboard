#include <stdio.h>
#include <stdint.h>

typedef enum {
  FDCAN_BYTES_0 = 0,
  FDCAN_BYTES_1 = 1,
  FDCAN_BYTES_2 = 2,
  FDCAN_BYTES_3 = 3,
  FDCAN_BYTES_4 = 4,
  FDCAN_BYTES_5 = 5,
  FDCAN_BYTES_6 = 6,
  FDCAN_BYTES_7 = 7,
  FDCAN_BYTES_8 = 8,
  FDCAN_BYTES_12 = 12,
  FDCAN_BYTES_16 = 16,
  FDCAN_BYTES_20 = 20,
  FDCAN_BYTES_24 = 24,
  FDCAN_BYTES_32 = 32,
  FDCAN_BYTES_48 = 48,
  FDCAN_BYTES_64 = 64
} fdcanBytes_t;

#define FDCAN_RELPACKMTR_ID 0x015
typedef struct {
  union {
    struct {
      uint32_t mtr_volt;
      uint32_t mtr_curr;
    };
    uint8_t FDCAN_RawRelPackMtr[FDCAN_BYTES_8];
  };
} FDCAN_RelPackMtr_t;

int main()
{
	FDCAN_RelPackMtr_t pack = {.mtr_volt = 0x12345678, .mtr_curr=0x01020304};
    printf("mtr_volt: %#06x\nmtr_curr: %#06x\n", pack.mtr_volt, pack.mtr_curr);
    for (int i = 0; i < sizeof(pack.FDCAN_RawRelPackMtr); i++){
    	printf("%#04x\n", pack.FDCAN_RawRelPackMtr[i]);
    }
}
