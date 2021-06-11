# panorama-explorer

Early developer upload: A panorama viewer powered by the 
[three-d](https://github.com/asny/three-d) rendering framework.

## Developer Setup
panorama-explorer stores its development media in the parallel
*panorama-explorer-dev-media* repo (to keep the main repo small).
Clone them to the same directory:
```
git clone https://github.com/kklibo/panorama-explorer
git clone https://github.com/kklibo/panorama-explorer-dev-media
```
### To build and run panorama-explorer in web mode:

To build:
```
cd panorama-explorer
wasm-pack build --target web --out-name web
```
To run, start `http-server` (or other http server) from the parent
directory (it must have access to the *panorama-explorer-dev-media*
repo directory).
```
cd ..
http-server
```
Open a browser, and load http://127.0.0.1:8080/panorama-explorer/index.html



### To build and run panorama-explorer in desktop mode:
```
cd panorama-explorer
cargo run
```

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
