# Polyhedra Expander SHA256 benchmark

The benchmark code is from Expander example: 
- https://github.com/PolyhedraZK/ExpanderCompilerCollection/blob/master/circuit-std-rs/tests/sha256_gf2.rs

## How to run
1. Enter the dir
```
cd polyhedra-expander
```

2. Run the script
```
chmod +x ./expander_proving.sh
./expander_proving.sh
```

### NOTE
1. Current sh scripts show correct performance metrics in only MacOS, not in ubuntu/linux.(Reason: `/usr/bin/time` util)
