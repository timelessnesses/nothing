# Nothing

Nothing, a fan made game to make fun of this [game](https://store.steampowered.com/app/2696480/Nothing) of how much space they use for just simple game.

## Building

Probably cross platform (Windows/Linux/Android) (Yes, fuck MacOS and iOS in general.)
For Android just switch to the android branch for detailed stuff

### Windows

1. install `cargo-vcpkg` with `cargo install cargo-vcpkg`
2. download sdl2 library for windows then put that into toolchain's library folder (info: [https://github.com/Rust-SDL2/rust-sdl2?tab=readme-ov-file#windows-msvc])
3. `cargo build -r` and magic

### Linux and everything else (Linux is tested while other isn't)

1. Install `libsdl2-dev` and `libsdl2-ttf-dev` and `libgtk-dev` (maybe not for macos for gtk)
2. Install `pkgconfig`
3. `cargo build -r`
