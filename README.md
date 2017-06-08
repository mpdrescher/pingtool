# pingtool
Ping one or multiple hosts. Simple frontend for https://github.com/cfallin/rust-oping

## Install

Install `liboping` from your package manager or from the repository (https://github.com/octo/liboping).  
Run `cargo build --release` for building, `cargo install` for installing.  
(You need to have Rust installed)  
  
On Linux this program needs root permissions!  
For more info see the liboping repo.
  
## Dependencies

- `https://github.com/octo/liboping` and its rust bindings `https://github.com/cfallin/rust-oping`
- `https://github.com/BurntSushi/tabwriter`
- `https://github.com/Stebalien/term`

## Screenshot

![Screenshot](/pingtool.png)