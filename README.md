# fpsdk

Rust port of [FL Studio SDK](https://www.image-line.com/developers/index.php).

The FL Studio SDK provides you the API libraries and developer tools necessary
to build, test, and debug plugins for FL Studio.




## Build


### Windows

You should enable Developer Mode. This is because
[cxx](https://crates.io/crates/cxx) should be able to create symlinks, what
is impossible without admin privileges or enabled Developer Mode on Windows.