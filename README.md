# nkts

The "experimental" multi-platform script execution engine for a certain vn game. Just for fun.
The aim of the project is to run ["Nukitashi"](https://qruppo.com/products/nukitashi/) and 
["Nukitashi 2"](https://qruppo.com/products/nukitashi2/) scripts (and hopefully other ShiinaRi*-based
games) on other platforms, such as macOS, Linux, Android, etc...

This is "experimental" i.e. quite far from the best.

## Prerequisites

* Computer with Vulkan capability
    * Checked on macOS Catalina 10.15.4; MacBook Pro (16-inch, 2019)
        * RAM: 16 GB
        * CPU: Intel Core i7-9750H
        * GPU (discrete): AMD Radeon Pro 5300M 4 GB
* Latest Rust stable compiler
* All assets extracted under `./blob/`
    * Use `arc-unpacker` or `GARbro`. We are lazy to implement the extraction/decryption methods.
    * images should be in `.S25` format so that the metadata is intact.
* A lot of patience
    * Seriously...

## Current status

For the command specification and the coverage, see [here](COMMANDS.md).

Some portions of scripts are now running on [`prototyping`](https://github.com/3c1u/nkts/tree/prototyping).
For prototyping, We use the [`prototyping`](https://github.com/3c1u/nkts/tree/prototyping) branch, which contains
a PoC for a script play-back using [Piston](https://github.com/PistonDevelopers/piston). We are going to
ditch Piston later and replace it with a Vulkan/winit-based engine, for the better compability and the portability.

We have no clue to *.SCR files, which seem to be responsible for the screen layout.
