file "target/thumbv7em-none-eabi/debug/coursework"

# Connect to texane stlink gdb server
target extended-remote :4242
# Or, connect to openOCD instead
# target remote localhost:3333

# monitor reset init
# monitor halt

# Uncomment to enable semihosting
# monitor arm semihosting enable

# Flash program and load symbols
load
break main

# Run to main (first breakpoint)
continue