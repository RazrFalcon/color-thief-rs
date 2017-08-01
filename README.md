## color-thief-rs

*color-thief-rs* is a [color-thief](https://github.com/lokesh/color-thief)
algorithm reimplementation in Rust.

The implementation itself is a heavily modified
[Swift version](https://github.com/yamoridon/ColorThiefSwift) of the same algorithm.

### Differences

- There is no `getColor` method, since it's [just a shorthand][color-thief_L76] for `getPalette`.
- Output colors are a bit different from JS version. See [tests](tests/test.rs) for details.

[color-thief_L76]: https://github.com/lokesh/color-thief/blob/b0115131476149500828b01db43ca701b099a315/src/color-thief.js#L76

### Performance

About 150x faster that JS version.

```text
test q1  ... bench:   1,429,800 ns/iter (+/- 21,987)
test q10 ... bench:     854,297 ns/iter (+/- 25,468)
```

### Usage

Dependency: [Rust](https://www.rust-lang.org/) >= 1.13

Add this to your `Cargo.toml`:

```toml
[dependencies]
color-thief = "0.1"
```

### License

*color-thief-rs* is licensed under the MIT.
