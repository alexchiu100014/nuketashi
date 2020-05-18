# nkts

The "experimental" multi-platform script execution engine for a certain vn game. Just for fun.
The aim of the project is to run ["Nukitashi"](https://qruppo.com/products/nukitashi/) and 
["Nukitashi 2"](https://qruppo.com/products/nukitashi2/) scripts (and hopefully other ShiinaRi*-based
games) on other platforms, such as macOS, Linux, Android, etc...

This is "experimental" i.e. quite far from the best.

## Screenshots

![ikuko](assets/screenshot_ikuko.png)
https://qruppo.com/

## Prerequisites

* Computer with Vulkan capability
    * Tested primarily on MacBook Pro (16-inch, 2019)
        * macOS Catalina 10.15.4
        * RAM: 16 GB
        * CPU: Intel Core i7-9750H
        * GPU (discrete): AMD Radeon Pro 5300M 4 GB
    * Also tested on ThinkPad X260 (but not frequently)
        * Fedora 32
        * RAM: 8GB
        * CPU: Intel Core i5-6200U
        * GPU (integrated): Intel HD Graphics 520
* Latest Rust stable compiler
* All assets extracted under `./blob/`
    * images should be in `.S25` format so that the metadata is intact.
* A lot of patience
    * Seriously...

## Current status

For the command specification and the coverage, see [here](COMMANDS.md).

Some portions of scripts are now running. ($L_CHR, $L_MONT, partially $A_CHR).
We have no clue to *.SCR files, which seem to be responsible for the screen layout.

## Disclaimer

This project is not affiliated to Qruppo, nor the developer(s) of ShiinaRio,
and is solely for educational and informational purposes.

## License

Copyright (c) 2020 Hikaru Terazono. All rights reserved.

This program is licensed under either MIT License or Apache License 2.0 at your option.
