[<img alt="github" src="https://img.shields.io/badge/github-dynotestpolije/dyno_app-8da0cb?logo=github" height="20">](https://github.com/dynotestpolije/dyno_app)
[![Build Status](https://github.com/dynotestpolije/dyno_types/workflows/CI/badge.svg)](https://github.com/dynotestpolije/dyno_types/actions?workflow=CI)
[![MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/dynotestpolije/dyno_types/blob/master/LICENSE)


<center>
    <h1>Dynotest App Desktop</h1>
    <p>desktop application for dynotest by Rizal Achmad Pahlevi</p>
</center>


## DEPENDENCIES

- compiler (rust)
    ```bash
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    ```
- system dependencies
    **(DEBIAN / UBUNTU)**
    - libxcb-render0
    - libxcb-shape0
    - libxcb-xfixes0
    - libxkbcommon
    - libssl
    ```bash
    sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libssl-dev
    ```

## INSTALATIONS

#### BUILD FROM SOURCE
**LINUX**
1. install dependencies [DEPENDENCIES](#dependencies)
2. clone or download the repo
```bash 
git clone --depth=1 'https://github.com/dynotestpolije/dyno_app.git'
cd dyno_app
```
3. build or run
```bash 
./check.sh 
cargo build --release 
# or you can run it
cargo run --release
```
4. install in system path
```bash 
cargo install
```
3. finish


