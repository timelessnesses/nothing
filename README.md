# Nothing

Nothing, a fan made game to make fun of this [game](https://store.steampowered.com/app/2696480/Nothing) of how much space they use for just simple game.

## Building

1. Install these toolchains `aarch64-linux-android armv7-linux-androideabi i686-linux-android`
2. Download SDL2 source code
3. Extract the SDL2 zip file then go to extracted location then build it with `ndk-build NDK_PROJECT_PATH=. APP_BUILD_SCRIPT=./Android.mk APP_PLATFORM=android-21`
4. Move `android-project` folder to the project folder
