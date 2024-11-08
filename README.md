# Website4Share

Website4Share is a Rust-based web application for sharing files and pasteboard content.

## Prerequisites

- Rust and Cargo installed
- Docker installed (for containerization)

## Running the Application with Cargo

To run the application using Cargo, follow these steps:

1. Clone the repository:

    ```sh
    git clone https://github.com/xz-dev/website4share.git
    cd website4share
    ```

2. Set the required environment variables:

    - `LISTEN_ADDR`: The address and port the application will listen on (default: `0.0.0.0:8080`).
    - `TMPDIR`: The directory for temporary files (default: system temporary directory).

    Example:

    ```sh
    export LISTEN_ADDR=0.0.0.0:8080
    export TMPDIR=/path/to/tempdir
    ```

3. Run the application:

    ```sh
    cargo run
    ```

## Building and Running with Docker

To build and run the application using Docker, follow these steps:

1. Build the Docker image:

    ```sh
    docker build -t website4share -f Containerfile .
    ```

2. Run the Docker container:

    ```sh
    docker run -d \
      -p 8080:8080 \
      -v /path/to/local/cache:/tmp/website4share \
      -e LISTEN_ADDR=0.0.0.0:8080 \
      --name website4share_container \
      website4share
    ```

### Explanation of Docker Run Command

- `-d`: Run the container in detached mode.
- `-p 8080:8080`: Map port 8080 on the host to port 8080 in the container.
- `-v /path/to/local/cache:/tmp/website4share`: Mount the local directory `/path/to/local/cache` to `/tmp/website4share` in the container. This ensures that the cache directory is persisted and not lost when the container is stopped or removed.
- `-e LISTEN_ADDR=0.0.0.0:8080`: Set the `LISTEN_ADDR` environment variable to `0.0.0.0:8080` to ensure the application listens on all network interfaces.
- `--name website4share_container`: Assign a name to the container for easier management.
- `website4share`: The name of the Docker image to run.

## Environment Variables

- `LISTEN_ADDR`: The address and port the application will listen on. Default is `0.0.0.0:8080`.
- `TMPDIR`: The directory for temporary files. Default is the system temporary directory + website4share.

## Project Structure

- `src/`: Source code of the application.
- `Cargo.toml`: Cargo configuration file.
- `Containerfile`: Dockerfile for building the Docker image.
- `static/`: Static files served by the application.

## Feature
1. Just a website
2. Multi share thread
3. Upload resume

## ScreenShot
- Home page:
  ![图片](https://github.com/user-attachments/assets/5a3a385f-2b87-4cce-a174-19cc0f897b88)
- Sub-page:
  ![图片](https://github.com/user-attachments/assets/0b266b5f-3bf1-417a-9287-b817abc25905)
  ![图片](https://github.com/user-attachments/assets/34dba68e-020a-48a8-8595-5b851516bf49)
