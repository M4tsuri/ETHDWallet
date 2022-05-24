/* see https://sciencezero.4hv.org/index.php?title=STM32F407_Microcontroller */
MEMORY
{
  /* Flash memory begins at 0x80000000 and has a size of 1MB*/
  FLASH (rx) : ORIGIN = 0x08000000, LENGTH = 512K
  /* sector 8 */
  DATA (rw)  : ORIGIN = 0x08080000, LENGTH = 128K
  /* RAM begins at 0x20000000 and has a size of 112kB*/
  RAM : ORIGIN = 0x20000000, LENGTH = 112K
}

SECTIONS
{
  .wallet : 
  {
    . = ALIGN(16);
    KEEP(*(.wallet));
    . = ALIGN(16);
  } > DATA
}
