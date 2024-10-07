# TTS-API-Server

This project is a RESTful API server that creates an audio from a text based on [Piper](https://github.com/rhasspy/piper). The APIs are compatible with OpenAI APIs of [create speech](https://platform.openai.com/docs/api-reference/audio/createSpeech).

> [!NOTE]
> The project is still under active development. The existing features still need to be improved and more features will be added in the future.

## Quick Start

### Setup

- Install WasmEdge v0.14.1

  ```bash
  curl -sSf https://raw.githubusercontent.com/WasmEdge/WasmEdge/master/utils/install_v2.sh | bash -s -- -v 0.14.1
  ```

- Deply `wasmedge-piper` plugin

  > For the purpose of demonstration, we will use the piper plugin for Ubuntu-20.04. You can find the plugin for other platforms [Releases/0.14.1](https://github.com/WasmEdge/WasmEdge/releases/tag/0.14.1)

  ```bash
  # Download piper plugin for Mac Apple Silicon
  curl -LO https://github.com/WasmEdge/WasmEdge/releases/download/0.14.1/WasmEdge-plugin-wasi_nn-piper-0.14.1-ubuntu20.04_x86_64.tar.gz

  # Unzip the plugin to $HOME/.wasmedge/plugin
  tar -xzf WasmEdge-plugin-wasi_nn-piper-0.14.1-ubuntu20.04_x86_64.tar.gz -C $HOME/.wasmedge/plugin

  rm $HOME/.wasmedge/plugin/libwasmedgePluginWasiNN.dylib
  ```

### Run tts-api-server

- Download piper model and voice config file

  ```bash
  # Download piper model
  curl -LO https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_US/lessac/medium/en_US-lessac-medium.onnx

  # Download voice config file
  curl -LO https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_US/lessac/medium/en_US-lessac-medium.onnx.json
  ```

  For more voice models and config files, visit [rhasspy/piper-voices](https://huggingface.co/rhasspy/piper-voices).

- Download text-to-speech synthesizer

  ```bash
  # Download espeak-ng data directory
  curl -LO https://github.com/rhasspy/piper/releases/download/2023.11.14-2/piper_linux_x86_64.tar.gz
  tar -xzf piper_linux_x86_64.tar.gz piper/espeak-ng-data --strip-components=1
  ```

- Download tts-api-server.wasm

  ```bash
  curl -LO
  ```

- Start server

  ```bash
  wasmedge --dir .:. tts-api-server.wasm
  ```

  > [!TIP]
  > `tts-api-server` will use `8080` port by default. You can change the port by adding `--port <port>`.

### Usage

- Send a request for creating an audio from a text

  ```bash
  curl --location 'http://localhost:8080/v1/audio/speech' \
    --header 'Content-Type: application/json' \
    --data '{
      "model": "piper",
      "input": "This is a audio speech test",
      "response_format": "wav",
      "speed": 1.0
    }'
  ```

  If the request is successful, you will receive a response like this:

  ```json
  {"id":"file_ee61934f-090f-4397-8e52-4ebb7d490c87","bytes":88224,"created_aename":"output.wav","object":"file","purpose":"assistants_output"}
  ```

  The generated "output.wav" file is located in the `./archives/file_ee61934f-090f-4397-8e52-4ebb7d490c87` directory.

## Build

- For **Linux users**

  ```bash
  cargo build --release
  ```

- For **macOS users**

  - Download the `wasi-sdk` from the [official website](https://github.com/WebAssembly/wasi-sdk/releases) and unzip it to the directory you want.

  - Build the project

    ```bash
    export WASI_SDK_PATH=/path/to/wasi-sdk
    export CC="${WASI_SDK_PATH}/bin/clang --sysroot=${WASI_SDK_PATH}/share/wasi-sysroot"
    cargo clean
    cargo update
    cargo build --release
    ```

If the build process is successful, `tts-api-server.wasm` will be generated in `target/wasm32-wasip1/release/`.

### CLI Options

```bash
$ wasmedge tts-api-server.wasm -h
Whisper API Server

Usage: tts-api-server.wasm [OPTIONS] --model-name <MODEL_NAME> --model <MODEL> --config <CONFIG> --espeak-ng-dir <ESPEAK_NG_DIR>

Options:
  -m, --model-name <MODEL_NAME>        Model name
      --model <MODEL>                  Path to the whisper model file
      --config <CONFIG>                Path to the voice config file
      --espeak-ng-dir <ESPEAK_NG_DIR>  Path to the espeak-ng data directory
      --socket-addr <SOCKET_ADDR>      Socket address of LlamaEdge API Server instance. For example, `0.0.0.0:8080`
      --port <PORT>                    Port number [default: 8080]
  -h, --help                           Print help
  -V, --version                        Print version
```