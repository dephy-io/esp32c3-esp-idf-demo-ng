```
# Deps
rustup toolchain install nightly --component rust-src
cargo install ldproxy
cargo install espflash --locked
cargo install cargo-espflash --locked

# Generate key
dd if=/dev/urandom of=dev_key.bin bs=32 count=1

# Burn key
espefuse.py burn_key BLOCK_KEY1 ./dev_key.bin USER --port /dev/cu.wchusbserial*

# Confirm key
espefuse.py summary --port /dev/cu.wchusbserial*

# before building
export PATH="$PATH:$PWD/.embuild/espressif/tools/riscv32-esp-elf/esp-13.2.0_20240530/riscv32-esp-elf/bin"

cp build.env.example build.env

```