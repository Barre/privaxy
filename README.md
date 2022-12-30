<div align="center">
  <img src="https://user-images.githubusercontent.com/45085843/146658168-a4770cf5-e1b1-40e6-8931-ffc64d3d4936.png">

  <h1>Privaxy</h1>

  <p>
    <strong>Next generation tracker and advertisement blocker</strong>
  </p>
</div>

<div align="center">
<img width="868" alt="dashboard" src="https://user-images.githubusercontent.com/45085843/210057822-f8a1e355-1b4d-4c48-a8c6-d72388e3b648.png">
<img width="912" alt="requests" src="https://user-images.githubusercontent.com/45085843/210057831-6c6b4aac-245c-4964-9d34-bcbd87d00a5f.png">
<img width="912" alt="filters" src="https://user-images.githubusercontent.com/45085843/210057827-9a413c82-77dd-4aaa-a7db-e12f0045608f.png">
<img width="912" alt="exclusions" src="https://user-images.githubusercontent.com/45085843/210057826-88168855-ac3e-4117-9d27-34199c39a7f3.png">
<img width="868" alt="custom_fiters" src="https://user-images.githubusercontent.com/45085843/210057820-d666baa5-4f63-45ca-ad2d-9eca95590100.png">
<img width="666" alt="taskbar" src="https://user-images.githubusercontent.com/45085843/210057833-df002cfd-aecf-4d67-bdd6-225ac3d6b980.png">
</div>

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

## Installation

### Using a pre-built binary

Pre-built binaries for Macos and Linux (x86_64) are provided on [github releases](https://github.com/Barre/privaxy/releases).

### Local system configuration

1. Go to the GUI, click on "Save CA certificate".
2. Install the downloaded certificate locally.
    - Macos: <https://support.apple.com/guide/keychain-access/add-certificates-to-a-keychain-kyca2431/mac>
    - Linux: `cp privaxy_ca_certificate.pem /usr/local/share/ca-certificates/`
3. Configure your local system to pass http traffic through privaxy.
   - Macos: <https://support.apple.com/guide/mac-help/change-proxy-settings-network-preferences-mac-mchlp2591/mac>
   - Ubuntu (gnome): <https://phoenixnap.com/kb/ubuntu-proxy-settings>
