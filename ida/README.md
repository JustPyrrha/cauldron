## Setting up IDA for Decima.


<details>
    <summary>Horizon: Forbidden West</summary>

1. Enable IDAClang
    - Options dropdown -> Compiler
      - Set Source parser to `clang`

2. Add Base RTTI Types
    - File dropdown -> Load File -> Parse C header file.
      - Select `HZR2_RTTI.hpp`

3. Generate Offsets IDC script
    - Launch HFW with `pulse.dll` installed via the IDA debugger.
      - Pulse must be built with the `debug_breakpoints` feature enabled.
    - Once the Offsets IDC is ready a software breakpoint will be hit.
    - It's recommended to wait for Ida finish indexing the file before proceeding.
    - File dropdown -> Script file
      - Select `<game>/cauldron/plugins/pulse/output/hfw_rtti.idc`.

</details>