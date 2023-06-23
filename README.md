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

## About Privaxy

Privaxy is a MITM HTTP(s) proxy that sits between HTTP(s) speaking applications, such as a web browser, and HTTP servers, such as those serving websites.

By establishing a two-way tunnel between the two ends, Privaxy is able to block network requests based on URL patterns and inject scripts and styles into HTML documents.

By operating at a lower level, Privaxy is both more efficient and more streamlined than browser add-on-based blockers. A single instance of Privaxy on a small virtual machine, server, or even on the same machine from which the traffic originates can filter thousands of requests per second while using very little memory.

Privaxy is not limited by browser APIs and can work with any HTTP traffic, not just traffic coming from web browsers.

Privaxy is also much more powerful than DNS-based blockers because it can operate directly on URLs and inject resources into web pages.

## Features

- Support for [Adblock Plus filters](https://adblockplus.org/filter-cheatsheet), such as [easylist](https://easylist.to/).
- Graphical web interface with statistics display and live request explorer.
- Support for uBlock origin's `js` syntax.
- Support for uBlock origin's `redirect' syntax.
- Support for uBlock origin's scriptlets.
- Browser and HTTP client agnostic.
- Support for custom filters.
- Support for excluding hosts from MITM pipeline.
- Support for protocol upgrades, such as websockets.
- Automatic filter list updates.
- Very low resource usage.
  - About 50MB of memory with about 320,000 filters enabled.
  - Capable of filtering thousands of requests per second on a small machine.

## Installation

### Using a pre-built binary

Pre-built binaries for major operating systems and platforms are provided at [github releases](https://github.com/Barre/privaxy/releases).

### Local system configuration

1. Go to the GUI, click on "Save CA certificate".
2. Install the downloaded certificate locally.
    - Macos: <https://support.apple.com/guide/keychain-access/add-certificates-to-a-keychain-kyca2431/mac>
    - Linux: `cp privaxy_ca_cert.pem /usr/local/share/ca-certificates/`
3. Configure your local system to pass http traffic through privaxy which listens on localhost:8100.
   - Macos: <https://support.apple.com/guide/mac-help/change-proxy-settings-network-preferences-mac-mchlp2591/mac>
   - Ubuntu (gnome): <https://phoenixnap.com/kb/ubuntu-proxy-settings>
