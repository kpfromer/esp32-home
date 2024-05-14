cargo build --release --features no-print && espflash flash -c esp32 -B 115200 --monitor target/xtensa-esp32-none-elf/release/transmitter_deep_sleep
