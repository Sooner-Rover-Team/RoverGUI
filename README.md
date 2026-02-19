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
~/RoverGUI/backend $ npm run start --host
```

From your laptop, **open the GUI in a web browser with its URL**: (i.e., open this link in Firefox/Chrome: [http://192.168.1.68:3000](http://192.168.1.68:3000), where `192.168.1.68` is the Rover's IP address, and `3000` is the port from the frontend window above; the port might be something different, so please look carefully). You should then be presented with a page with a dropdown list of available cameras. Select a camera -- then, a live stream should be visible!

## Dependencies

The following dependencies are under the assertion that both the frontend and backend are running on an **x86-64 Linux operating system**. Each bullet is a link to the corresponding dependency's download/install page.

### Frontend

- [NodeJS](https://nodejs.org/en/download)
- [NPM](https://docs.npmjs.com/downloading-and-installing-node-js-and-npm)

> [!TIP]
> It is recommended to install NodeJS using the **nvm** package manager on linux. This simplifies the process of both installing and upgrading NodeJS versions. This also automatically installs NPM for you! The NodeJS download page should provide instructions given that **Linux**, **nvm**, and **npm** are selected.

### Backend

- [Rust](https://rust-lang.org/tools/install/)
- Video4Linux (v4l)

> [!IMPORTANT]
> Video4Linux is a kernel-level driver dependency. It is usually included with most Linux distros. If it isn't, please consult your distro's documentation for installation instructions!

## Instructions

### Frontend

#### Installing Frontend's Package Dependencies

Before running the frontend, you will need to install the package's dependencies using your favorite terminal!

```bash
# First make sure that you are in the frontend's folder (/react-app)
cd react-app/

# Now install the dependencies!
npm i

# Your output should look something like this:
added 1472 packages, and audited 1473 packages in 14s

262 packages are looking for funding
  run `npm fund` for details

28 vulnerabilities (6 low, 8 moderate, 13 high, 1 critical)

To address issues that do not require attention, run:
  npm audit fix

To address all issues (including breaking changes), run:
  npm audit fix --force

Run `npm audit` for details.
npm notice
npm notice New major version of npm available! 10.9.4 -> 11.6.2
npm notice Changelog: https://github.com/npm/cli/releases/tag/v11.6.2
npm notice To update run: npm install -g npm@11.6.2
npm notice

# Done!
```

#### Running the frontend

```bash
# To start the frontend, simply run the following command in the frontend's folder.
npm run start --host

# Your output should look like this:
VITE v7.3.1  ready in 96 ms

➜  Local:   http://localhost:5173/
➜  Network: http://192.168.1.68:5173/
➜  press h + enter to show help
```

The URLs that are outputted by this command (i.e. http://localhost:3000) are the same URLs that you use to see the GUI from your web browser. You may hold ctrl and then click on the links to view them or simply copy & paste them into your browser.

> [!TIP]
> If the `npm run start` command fails, it may be because something else (like another instance of the frontend) is running on the frontend's same port (`3000` in this case). See the code block below for an example of this!

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
