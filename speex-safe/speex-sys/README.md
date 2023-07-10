# speex-sys

Unsafe direct bindings to the `speex` audio compression library.

## `speex` Changes

`speex` has been modified slightly from the original package.

No actual code has been changed. All `speex` .c and .h files are unmodified.

The source for the example `speexenc` and `speexdec` applications has been removed to cut down on package size, as it
is not utilized for the build.

All build files have been removed, since the project is compiled via `cc` rather than via provided build systems.

`speex_config_types.h` is included rather than generated as a consequence of this. It is generated with Linux defaults.

## License

`speex-sys` consists of the code used to generate bindings and is licensed under the terms of MPL-2.0. `speex-sys` files
include an MPL-2.0 header to make this distinction clear.

`speex` is the backing library, and is licensed under a 3 clause BSD style license. Its terms can be found in the
`speex` folder within the `COPYING` file.