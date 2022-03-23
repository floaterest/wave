# Wave
> Generate `.wav` file from user input

# Usage
- from executable: `wave input.txt output.wav`
- from source: `cargo run input.txt output.wav`

<details><summary>Input example:</summary>

the following staff:

![](/assets/ddrive.png)

<details><summary>can be transcribed as: <code>input.txt</code></summary>

```
140
==== bar 1 ====
	8 d5 d4
	8 a4 a3
	8 d5 d4
	8 g4 g3
	8 d5 d4
	8 f4 f3
	8 e4 e3
	8 c5 c4
==== bar 2 ====
	8 e4 e3
	8 f4 f3
	8 c5 c4
	8 f5 f4
	8 e5 e4
	8 c5 c4
	8 g4 g3
	8 c5 c4
==== bar 3 ====
	8 e4 e3
	8 f4 f3
	8 c5 c4
	8 f5 f4
	8 g5 g4
	8 e5 e4
	8 c5 c4
	16 e5
	16 f5
==== bar 4 ====
	8 e5 c5
	8 c5 g4
	8 g4 e4
	8 d5 a4
	8 a4 f4
	8 f4 d4
	8 c5 c4
	8 e4 e3
```
</details>

and the output will be:

https://user-images.githubusercontent.com/56704092/159800609-c127c967-e6b6-4a3d-b443-045ae1330b33.mp4

</details>

# Programmer's Note
- [`note.rs`](/src/note.rs)
    - convert note to key number to frequency
- [`wave.rs`](/src/wave.rs)
    - `.wav` file structure
    - generate sine wave
    - convert literally anything into bytes (`[u8]`) using unsafe `transmute()`