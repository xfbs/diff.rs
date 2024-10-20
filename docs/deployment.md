# Deployment

The workflow for deployment work like this:

- Building the frontend using `trunk build --release`
- Compressing assets using `gzip` and `brotli`
- Adding a `_redirects` file to configure it as a single-page application

![Deployment diagram](diagrams/deployment.svg)

Building is done using [Trunk][trunk]. It uses Cargo under the hood to compile
the code for `wasm32-unknown-unknown`, adds some glue code to launch it in the
browser and links the assets. Running this includes:

- Running `tailwindcss` to generate the Tailwind CSS styles.
- Using [wasm-bindgen][] to generate bindings for the WebAssembly binary for the browser.
- Using [wasm-opt][] to optimize the resulting WebAssembly binary.
- Including the assets and hashing them for Sub-Resource Integrity and cache busting
- Minifying the assets

Deployment is done using [GitLab CI][gitlab-ci]. The
[configuration](../.gitlab-ci.yml) runs Trunk, and uses `gzip` and `brotli` to
pre-compress the files. Using compression makes the resulting WebAssembly blob
and assets significantly smaller, at the time of writing, it makes it go from
3MB to 1MB. This step consists of:

Hosting of the project is provided by [GitLab Pages][gitlab-pages]. The
resulting files of the building and compression are statically hosted by it.
GitLab Pages will serve the precompressed files if the request indicates that
the client supports it. A `_redirects` file is used to tell it to serve the
`index.html` file for every route, to make it work as a single-page
application.

[trunk]: https://trunkrs.dev/
[gitlab-pages]: https://docs.gitlab.com/ee/user/project/pages/
[gitlab-ci]: https://docs.gitlab.com/ee/ci/
[wasm-opt]: https://github.com/WebAssembly/binaryen
[wasm-bindgen]: https://github.com/rustwasm/wasm-bindgen
