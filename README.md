# fpsdk

Rust port of [FL Studio SDK](https://www.image-line.com/developers/index.php).

The FL Studio SDK provides you the API libraries and developer tools necessary
to build, test, and debug plugins for FL Studio.




## Example

The example demonstrates how to use this library.

To build it, run:

```
cargo build --release --example simple
```

To install it:

```
./install.mac.sh simple Simple -g # for macOS
./install.win.bat simple Simple -g # for Windows
```

Check out the corresponding script for your system for usage notes.

The plugin's log file is created at FL's resources root. It's `/Applications/FL
Studio 20.app/Contents/Resources/FL` for macOS and `<Drive>:\Program
Files\Image-Line\FL Studio 20` for Windows.
