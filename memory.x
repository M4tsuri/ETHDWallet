/* see https://sciencezero.4hv.org/index.php?title=STM32F407_Microcontroller */
MEMORY
{
  /* Flash memory begins at 0x80000000 and has a size of 64kB*/
  FLASH : ORIGIN = 0x08000000, LENGTH = 1M
  /* RAM begins at 0x20000000 and has a size of 20kB*/
  RAM : ORIGIN = 0x20000000, LENGTH = 112K
}