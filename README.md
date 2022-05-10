<div align="center">
  <img src="https://user-images.githubusercontent.com/45085843/146658168-a4770cf5-e1b1-40e6-8931-ffc64d3d4936.png">

  <h1>Privaxy</h1>

  <p>
    <strong>Next generation tracker and advertisement blocker</strong>
  </p>
</div>

<img width="1481" alt="Screenshot 2022-05-09 at 22 00 42" src="https://user-images.githubusercontent.com/45085843/167488210-fd6df5f8-92e9-42c8-9170-3d17dc544862.png">
<img width="1497" alt="Screenshot 2022-05-09 at 22 01 27" src="https://user-images.githubusercontent.com/45085843/167488326-d585f306-0fdd-40b1-befa-441d6e6e353a.png">
<img width="1280" alt="Screenshot 2022-05-09 at 22 01 55" src="https://user-images.githubusercontent.com/45085843/167488384-7a19343d-5ef7-4d90-9a98-12743ef98ee0.png">
<img width="1276" alt="Screenshot 2022-05-09 at 22 02 09" src="https://user-images.githubusercontent.com/45085843/167488399-f9dc5e31-07d7-4709-9e15-ff8112d3c584.png">
<img width="1276" alt="Screenshot 2022-05-09 at 22 02 20" src="https://user-images.githubusercontent.com/45085843/167488522-b09fc22b-fff1-48ff-b471-4ace4f7ab995.png">

## Installation

### Disclaimer

This is an early release without authentication on the management API. You should proceed with caution and not expose the service on the internet. To prevent accidental exposure of the service, it's only yet possible to bind on `127.0.0.1`.

### Using a pre-built binary

Pre-built binaries for Macos and Linux (x86_64) are provided on [github releases](https://github.com/Barre/privaxy/releases).

### Using the rust toolchain

1. Begin by [installing rust](https://www.rust-lang.org/tools/install).
2. [Install trunk](https://trunkrs.dev/#install).
3. Clone this repository.
4. Build the web gui by running `cd web_frontend && trunk build --release && cd ..`
5. Build the server by running `cd privaxy && cargo build --release.`
6. Run privaxy using `cargo run --release --bin privaxy`.

### Local system configuration

1. Navigate to the web gui at `http://127.0.0.1:8000`, click on "Download CA certificate".
2. Install the downloaded certificate locally.
    - Macos: <https://support.apple.com/guide/keychain-access/add-certificates-to-a-keychain-kyca2431/mac>
    - Linux: `cp privaxy_ca_certificate.pem /usr/local/share/ca-certificates/`
3. Configure your local system to pass http traffic through privaxy.
   - Macos: <https://support.apple.com/guide/mac-help/change-proxy-settings-network-preferences-mac-mchlp2591/mac>
   - Ubuntu (gnome): <https://phoenixnap.com/kb/ubuntu-proxy-settings>

## About

Privaxy is a MITM HTTP(s) proxy that sits in between HTTP(s) talking applications, such as a web browser and HTTP servers, such as those serving websites.

By establishing a two-way tunnel between both ends, Privaxy is able to block network requests based on URL patterns and to inject scripts as well as styles into HTML documents.

Operating at a lower level, Privaxy is both more efficient as well as more streamlined than browser add-on-based blockers. A single instance of Privaxy on a small virtual machine, server or even, on the same computer as the traffic is originating from, can filter thousands of requests per second while requiring a very small amount of memory.

Privaxy is not limited by the browser’s APIs and can operate with any HTTP traffic, not only the traffic flowing from web browsers.

Privaxy is also way more capable than DNS-based blockers as it is able to operate directly on URLs and to inject resources into web pages.

## Features

- Suppport for [Adblock Plus filters](https://adblockplus.org/filter-cheatsheet), such as [easylist](https://easylist.to/).
- Web graphical user interface with a statistics display as well as a live request explorer.
- Support for uBlock origin's `js` syntax.
- Support for uBlock origin's `redirect` syntax.
- Support for uBlock origin's scriptlets.
- Browser and HTTP client agnostic.
- Support for custom filters.
- Support for excluding hosts from the MITM pipeline.
- Support for protocol upgrades, such as with websockets.
- Automatic filter lists updates.
- Very low resource usage.
  - Around 50MB of memory with approximately 320 000 filters enabled.
  - Able to filter thousands of requests per second on a small machine.
