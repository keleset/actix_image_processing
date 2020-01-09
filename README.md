# Actix image processing example
This project shows how image upload process can be organized on a webserver built with Actix-Web using Rust.
Both remote upload (as client) and multipart upload (as server) are implemented.

## How it works
This is a simple server implementation that has two following REST methods:
* `POST` `/upload/local` -- accepts multipart form upload (with local image files).
* `POST` `/upload/remote` -- accepts a `application/json` with array called `urls`, which contains urls of images intended to be uploaded to the server.

* For both of the methods above:
    * Uploaded images are stored on the server and thumbnail preview is being created for each. 
    * Returned type is `application/json` that contains info about what images were processed (uploaded) and their respective sizes (for full image and for thumbnail), or just the text of the error.
    * Several images can be processed with one request.

There are also:
* `GET` `/` -- returns simple webpage that demonstrates REST methods mentioned above.
* `GET` `/image/original/*` -- returns images stored on the server, where `*` should be replaced with image file name (that returned in json by POST methods above).
* `GET` `/image/preview/*` -- same as the one above, but returns `100x100` thumbnails.

## Quickstart
Webserver can be started on `127.0.0.1:30243` using command:
```
cargo run
```
After that, you may use `127.0.0.1:30243` in your web browser to test it around yourself.

## Test
There is also a single integration test what uploads several imaged from remote urls and tests if they being returned by the GET `/image/*` methods mentioned above. Can be executed with:
```
cargo test
```

## Known issues
Server have to be executed in a directory that allows read/write access to successfully store images.
Client SSL part of the Actix-Web framework has a bug that can cause problems on some Windows systems (typically on those, which have several network interfaces) -- very long name resolution. Typically that results in Timeout for outgoing connections.
