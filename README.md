[![Github Link][github badge]][github link]
[![MSRV][rust 1.51.0+ badge]][rust 1.51.0+ link]
![license]

<br />
<h1 align="center">Rustcraft</h1>
  <p align="center">
    A simple Minecraft engine <i>in Rust</i>
  </p>
  <p align="center">
      <img src="photos/world_example1.png" alt="Logo" >
  </p>
</p>

## About The Project
Welcome to RUSTCRAFT!
Rustcraft is a simple Minecraft engine written in rust using [wgpu](https://github.com/gfx-rs/wgpu-rs).

### But Why?

I wanted to work on a project to learn more about [wgpu](https://github.com/gfx-rs/wgpu-rs) as well as practice my Rust skills. I then randomly saw this youtube [video](https://youtu.be/4O0_-1NaWnY) and got inspired to try to make a simple Minecraft engine in Rust!

## Inspiration

As I explained I was inspired by this [video](https://youtu.be/4O0_-1NaWnY) and the corresponding [codebase](https://github.com/jdah/minecraft-weekend). This is why you will probably see a lot of design similarities between my project and [this](https://github.com/jdah/minecraft-weekend) one. [Jdah](https://github.com/jdah), your video was awesome. (I also copied his blocks atlas because I liked it a lot).

## Current State

I am pretty happy with the current state of the engine. There are a few problems with it but they are outside of my MVP scope. I am aware that the game is not as efficient and smooth as it should be. I initially set as a goal to not use any threading or unsafe rust. I did end up eventually using [rayon](https://github.com/rayon-rs/rayon) (but I should really be using [Tokio](https://github.com/tokio-rs/tokio)). This would require some not trivial redesign and maybe I will update this in the future 🤷.

The [noise](https://github.com/razaekel/noise-rs) library I am using is also a bit slow. There are some issues with integrating it nicely into a bigger project but I don't really want to write my own noise library right now so this works.

## Potential Updates

- [ ] Transparent water
- [ ] Lighting (real sun or "hacked" lighting)
- [ ] Block memory map (saves location of broken/placed blocks)
- [ ] Game physics
- [ ] additional sprites: (tree, flowers)
- [ ] Block selection during placement
- [ ] Biomes
- [ ] "Real" concurency for chunk generation (Tokio).

## Prerequisites

- [rust](https://www.rust-lang.org/learn/get-started)

## Running

```bash
cargo run --release
```

### Arguments

Running in wireframe mode:

```bash
cargo run --release -- -w
```

Running with coordinate system at [0,0,0] (for debugging):

```bash
cargo run --release -- -c
```

Running a flat world:

```bash
cargo run --release -- -f
```

### Controls

- `awsd` to move around
- `space` move upwards
- `shift` move down
- `left click` break block
- `right click` place block

## Authors

- **Raphael Van Hoffelen** - [website](www.raphaelvanhoffelen.com)

## License

This project is licensed under the [MIT](LICENSE) - see the [LICENSE](LICENSE) file for
details.

[github badge]: https://img.shields.io/badge/github-dskart/rustcraft-8da0cb?style=flat&logo=github
[github link]: https://github.com/dskart/rustcraft
[rust 1.51.0+ badge]: https://img.shields.io/badge/rust-1.51.0+-93450a.svg?style=flat&logo=rust
[rust 1.51.0+ link]: https://blog.rust-lang.org/2021/03/25/Rust-1.51.0.html
[license]: https://img.shields.io/github/license/dskart/rustcraft?style=flat 