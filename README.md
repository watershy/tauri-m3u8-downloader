# 🎬 M3U8 Downloader

A blazing-fast, cross-platform desktop application for downloading and merging M3U8 video streams. Built with **Tauri**, **Rust**, and **Vue.js**.

## Demo

### Standard Download
https://github.com/user-attachments/assets/2e76340b-7372-4f0a-927a-2a6964390dd0

### Bypassing 403 Errors (Custom Headers)
Many streaming servers protect their videos by requiring specific `Referer` or `User-Agent` headers. M3U8 Downloader lets you inject custom HTTP headers directly into the request to bypass these restrictions and download previously inaccessible streams.

https://github.com/user-attachments/assets/7a5cdf05-b063-41e0-b2d6-4f5394c35502

## ✨ Features
* **High-Speed Concurrency:** A Rust backend downloads multiple video segments simultaneously to maximize your network bandwidth.
* **Custom HTTP Headers:** Inject `Referer`, `Cookie`, or `User-Agent` headers to bypass 403 Forbidden errors on protected streams.
* **On-the-Fly Decryption:** Automatically detects `#EXT-X-KEY` tags, fetches the keys, and decrypts AES-128 scrambled segments directly in memory.
* **Auto-Merge:** Automatically stitches `.ts` and `.m4s` segments into a single, clean .mp4 file using FFmpeg.
* **Pause & Resume:** Safely halt active downloads and resume them later without losing any downloaded data.
* **Error Recovery:** Built-in retry logic automatically handles network hiccups and dropped segments.
* **Responsive UI:** A clean Vue frontend featuring real-time progress tracking and exponentially smoothed ETAs.

## 🚀 Installation (Windows)

The easiest way to use the app is to download the pre-compiled installer.

1. Go to the [Releases](https://github.com/watershy/tauri-m3u8-downloader/releases) page.
2. Download the latest `M3U8Downloader_setup.exe`.
3. Run the installer (You can choose to install for just yourself, or all users).
4. *Note: You must have FFmpeg installed and added to your system PATH for the final merge step to work.*

## 🛠️ Development Setup

Want to build it yourself or contribute? Here is how to run the project locally.

### Prerequisites
* [Node.js](https://nodejs.org/)
* [Rust](https://www.rust-lang.org/tools/install)
* [FFmpeg](https://ffmpeg.org/download.html)

### Running Locally

1. Clone the repository:
   git clone https://github.com/watershy/tauri-m3u8-downloader.git
   cd tauri-m3u8-downloader

2. Install frontend dependencies:
   cd ui
   npm install

3. Run the development server:
   cargo tauri dev

### Building for Production
To build the final NSIS installer:
   cd ui && npm run build
   cd ..
   cargo tauri build

The compiled installer will be located in `src-tauri/target/release/bundle/nsis/`.

## 📄 License
This project is licensed under the MIT License.
