# STWO(cairo-lang) SHA256 benchmark

The benchmark codes are based on: 
https://github.com/cartridge-gg/cairo-sha256/blob/main/src/sha256.cairo 
https://github.com/cartridge-gg/cairo-sha256/blob/main/src/packed_sha256.cairo 
https://github.com/cartridge-gg/cairo-sha256/blob/main/tests/test_sha256.cairo 

## How to run
1. Install the [`uv`](https://github.com/astral-sh/uv) tool
2. Enter the dir
```
cd stwo
```
3. Setup the environment
```
chmod +x setup.sh
./setup.sh
```
3. Try to run proving with STWO prover
```
chmod +x stwo_proving.sh
./stwo_proving.sh
```
4. Try to run verifying with STWO prover(WIP)
```
chmod +x stwo_verify.sh
./stwo_verify.sh
```
## References
- https://github.com/cartridge-gg/cairo-sha256
- https://github.com/starkware-libs/stwo-cairo?tab=readme-ov-file#using-stwo-to-prove-cairo-programs
- https://docs.cairo-lang.org/cairozero/quickstart.html

## NOTE
1. Current sh scripts show correct performance metrics in only MacOS, not in ubuntu/linux.(Reason: `/usr/bin/time` util)
2. `stwo_verify.sh` file is not working atm. It outputs some weird error when verifying.
