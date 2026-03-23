# RoverGUI

RoverGUI provides a live camera view to anyone operating the rover via a web browser! Non-Autonomous members will likely be using the GUI, so it's easy to use.

## Usage

First, open two `ssh` windows to the Rover. Run both the backend and frontend, starting **with the backend first**:

In window 1:

```console
~ $ cd RoverGUI/backend
~/RoverGUI/backend $ cargo run
```

In window 2:

```console
~ $ cd RoverGUI/react-app
~/RoverGUI/react-app $ pnpm start --host
```

From your laptop, **open the GUI in a web browser with its URL**: (i.e., open this link in Firefox/Chrome: http://192.168.1.68:5173, where `192.168.1.68` is the Rover's IP address, and `5173` is the port from the frontend window above; the port might be something different, so please look carefully). You should then be presented with a page with a dropdown list of available cameras. Select a camera -- then, a live stream should be visible!

## Dependencies

You **must** download and install the following dependencies:

- `Node.js`: https://nodejs.org/en/download
- `pnpm`: https://pnpm.io/installation
- Rust (`cargo`): https://rustup.rs/

If you're running Linux, and you want to use the real backend (i.e., you'd like to stream real video from real cameras), you should also install `Video4Linux` on your Linux distro. See: https://pkgs.org/search/?q=v4l-utils

## Detailed Instructions

### Frontend

#### Installing Frontend's Package Dependencies

Before running the frontend, you will need to install the package's dependencies using your favorite terminal!

```bash
# First make sure that you are in the frontend's folder (/react-app)
cd react-app/

# Now install the dependencies!
pnpm install

# Done!
```

#### Running the frontend

```bash
# To start the frontend, simply run the following command in the frontend's folder.
pnpm start --host

# Your output should look like this:
VITE v7.3.1  ready in 96 ms

➜  Local:   http://localhost:5173/
➜  Network: http://192.168.1.68:5173/
➜  press h + enter to show help
```

The URLs that are outputted by this command (i.e. http://localhost:5173) are the same URLs that you use to see the GUI from your web browser. You may hold ctrl and then click on the links to view them or simply copy & paste them into your browser.

> [!TIP]
> If the `pnpm start` command fails, it may be because something else (like another instance of the frontend) is running on the same port (`5173` in this case). Vite may prompt you to use a different port or automatically switch ports.

#### Stopping the frontend

In the same terminal in which you are running the frontend, press `Ctrl+C` to send a SIGINT signal to interrupt the frontend (this stops it)!

### Backend (`/backend`)

#### Building the backend crate and its Cargo dependencies

Before running the backend, you will need to build the backend along with all of its dependencies.

```bash
# First make sure that you are in the backend's folder (/backend)
cd backend/

# Build the backend and dependencies
cargo build

# Done!
```

## Running the backend

```bash
# Simply run:
cargo run

# Done!
```

The output of this command tells important information about the backend like its address, port, and its available routes. In this case, the frontend would need to access the backend from the URL `http://127.0.0.1:3600`.

### Stopping the backend

In the same terminal in which you are running the backend, simply do `Ctrl+C` to send a SIGINT signal to interrupt the backend (this stops it)!

### The "fake" backend

There's a `fake_backend` binary you can use to test the frontend on a non-Linux computer (i.e., macOS or Windows). To use it, just type:

```bash
NUM_CAMERAS=1 cargo run --bin fake_backend $NUM_CAMERAS
```

You can set `NUM_CAMERAS` to any value you'd like.

Then, open up the frontend as described above. It'll connect successfully!

You can press `Ctrl+C` to stop the fake backend from running.
