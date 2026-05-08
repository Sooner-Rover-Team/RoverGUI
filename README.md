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

From your laptop, **open the GUI in a web browser with its URL**: (i.e., open this link in Firefox/Chrome: [http://192.168.1.68:3000](http://192.168.1.68:3000), where `192.168.1.68` is the Rover's IP address, and `3000` is the port from the frontend window above; the port might be something different, so please look carefully). You should then be presented with a page with a dropdown list of available cameras. Select a camera -- then, a live stream should be visible!

## Dependencies

You **must** download and install the following dependencies:

- `Node.js`: [Download Node.js](https://nodejs.org/en/download)
- `pnpm`: [Downloading and installing `pnpm`](https://pnpm.io/installation)
- Rust (`cargo`): [An installer for the systems programming language Rust](https://rustup.rs/)

If you're running Linux, and you want to use the real backend (i.e., you'd like to stream real video from real cameras), you should also install `Video4Linux` on your Linux distro. See: [pkgs.org - v4l-utils](https://pkgs.org/search/?q=v4l-utils).

## Detailed Instructions

### Frontend

#### Installing Frontend's Package Dependencies

Before running the frontend, you will need to install the package's dependencies using your favorite terminal!

```bash
# First make sure that you are in the frontend's folder (/react-app)
cd react-app/

# Now install the dependencies!
pnpm install

# Your output should look something like this, if installing for the first time:
Packages: +345
++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
Progress: resolved 345, reused 0, downloaded 345, added 345, done

dependencies:
+ @testing-library/jest-dom 5.17.0
+ @testing-library/react 16.3.2
+ @testing-library/user-event 13.5.0
+ @types/react 19.2.14
+ @types/react-dom 19.2.3
+ react 19.2.4
+ react-dom 19.2.4
+ react-player 2.16.1
+ web-vitals 2.1.4

devDependencies:
+ @biomejs/biome 2.3.8
+ @vitejs/plugin-react 5.2.0
+ jsdom 28.1.0
+ source-map-loader 5.0.0
+ ts-loader 9.5.4
+ typescript 5.9.3
+ vite 7.3.1
+ vitest 4.1.0

Done in 12.3s using pnpm v10.32.1


# Your output should look something like this, if already installed:
Lockfile is up to date, resolution step is skipped
Already up to date

   ╭─────────────────────────────────────────╮
   │                                         │
   │   Update available! 10.32.1 → 11.0.8.   │
   │   Changelog: https://pnpm.io/v/11.0.8   │
   │    To update, run: pnpm self-update     │
   │                                         │
   ╰─────────────────────────────────────────╯

Done in 862ms using pnpm v10.32.1
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

The URLs that are outputted by this command (i.e. http://localhost:3000) are the same URLs that you use to see the GUI from your web browser. You may hold ctrl and then click on the links to view them or simply copy & paste them into your browser.

> [!TIP]
> If the `pnpm start` command fails, it may be because something else (like another instance of the frontend) is running on the frontend's same port (`3000` in this case). See the code block below for an example of this!

```bash
? Something is already running on port 3000. Probably:
  /home/mizael/.nvm/versions/node/v22.21.0/bin/node /home/mizael/RoverGUI/react-app/node_modules/react-scripts/scripts/start.js (pid 67442)
  in /home/mizael/RoverGUI/react-app

Would you like to run the app on another port instead? › (Y/n)
```

#### Stopping the frontend

In the same terminal in which you are running the frontend, press `Ctrl+C` to send a SIGINT signal to interrupt the frontend (this stops it)!

### Backend (`/backend`)

#### Building the backend crate and its Cargo dependencies

Before running the backend, you will need to build the frontend along with all of its dependencies.

```bash
# First make sure that you are in the backend's folder (/backend)
cd backend/

# Cargo makes the process of building the crate really easy! Simply run the following command which will fetch and compile all of the dependencies along with building the backend package!
cargo build

# The output should look something like this:
   Compiling fastrand v2.3.0
   Compiling spin v0.9.8
   Compiling mime v0.3.17
   Compiling async-stream v0.3.6
   Compiling tempfile v3.23.0
   Compiling webrtc-data v0.9.0
   Compiling webrtc-ice v0.11.0
   Compiling rocket_codegen v0.5.1
   Compiling webrtc-dtls v0.10.0
   Compiling sdp v0.6.2
   Compiling rayon v1.11.0
   Compiling interceptor v0.12.0
   Compiling wide v0.7.33
   Compiling webrtc-media v0.8.0
   Compiling tokio-stream v0.1.17
   Compiling smol_str v0.2.2
   Compiling ubyte v0.10.4
   Compiling num_cpus v1.17.0
   Compiling atomic v0.5.3
   Compiling binascii v0.1.4
   Compiling hex v0.4.3
   Compiling jpeg-decoder v0.3.2
   Compiling v4l v0.14.0
   Compiling webrtc v0.11.0
   Compiling openh264 v0.6.6
   Compiling backend-rs v0.1.0 (/home/mizael/RoverGUI/backend)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 49.80s

# Done!
```

## Running the backend

```bash
# Simply run:
cargo run

# Your output should look like this:
arget(s) in 0.24s
     Running `target/debug/backend-rs`
🔧 Configured for debug.
   >> address: 127.0.0.1
   >> port: 3600
   >> workers: 12
   >> max blocking threads: 512
   >> ident: Rocket
   >> IP header: X-Real-IP
   >> limits: bytes = 8KiB, data-form = 2MiB, file = 1MiB, form = 32KiB, json = 1MiB, msgpack = 1MiB, string = 8KiB
   >> temp dir: /tmp
   >> http/2: true
   >> keep-alive: 5s
   >> tls: disabled
   >> shutdown: ctrlc = true, force = true, signals = [SIGTERM], grace = 2s, mercy = 3s
   >> log level: normal
   >> cli colors: true
📬 Routes:
   >> (get_available_cameras) GET /stream/cameras
   >> (get_camera_feed) POST /stream/cameras/<camera_path>/start
   >> (get_camera_modes) GET /stream/cameras/<camera_path>/modes
   >> (get_camera_mode) GET /stream/cameras/<camera_path>/modes/current
   >> (get_camera_mode_set) GET /stream/cameras/<camera_path>/modes/set/<mode_id>
📡 Fairings:
   >> Shield (liftoff, response, singleton)
🛡️ Shield:
   >> X-Content-Type-Options: nosniff
   >> X-Frame-Options: SAMEORIGIN
   >> Permissions-Policy: interest-cohort=()
🚀 Rocket has launched from http://127.0.0.1:3600

# Done!
```

The output of this command tells important information about the backend like its address, port, and its available routes. In this case, the frontend would need to access the backend from the URL `http://127.0.0.1:3600`.

### Stopping the backend

In the same terminal in which you are running the backend, simply do `Ctrl+C` to send a SIGINT signal to interrupt the backend (this stops it)!

### The "fake" backend

There's a `fake_backend` binary you can use to test the frontend on a non-Linux computer (i.e., macOS or Windows). To use it, just type: `NUM_CAMERAS=1 cargo run --bin fake_backend $NUM_CAMERAS`. You can set `NUM_CAMERAS` to any value you'd like.

Then, open up the frontend as described above. It'll connect successfully!

You can press `Ctrl+C` to stop the fake backend from running.
