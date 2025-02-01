```
# Install dependencies
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

# Remenber to edit build.env
cp build.env.example build.env

# Build and run (debug profile)
cargo run

# Build and flash (release profile)
cargo espflash flash --release
```