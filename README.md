![big](https://s11.gifyu.com/images/S1fVw.gif)

Build with `cargo build --release` ([cargo](https://www.rust-lang.org/fr/learn/get-started) needs to be installed)

Usage: `target/release/big TEXT [options]` (piping in from other programs works as well)

Options:
```
    -f, --font PATH     set font
    -s, --speed INT     set text speed (default: 10)
    -S, --size INT      set text size (default: 10)
    -e, --effects EFFECT1 EFFECT2 ...
                        pick only some effects
    -l, --loop          loop text
    -h, --help          print help menu
```

Background effects :
 - empty
 - wave
 - spiral

Text effects :
 - normal
 - rainbow
 - split
 - worm