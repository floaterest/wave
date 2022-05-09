# Note Token
> let's see how many ASCII characters I can adopt for each possible dynamic/articulation/ornament/note value and note relationship

- reference: [parsers/notes.rs](../src/parsers/note.rs)

## Frequency Token
> uses HashMap to get the frequency at constant time and doesn't stores each frequency in cache

- uses [scientific pitch notation](https://en.wikipedia.org/wiki/Scientific_pitch_notation) but in lowercase

### Examples
- `c5`: Tenor C
- `a4`: A440
- `eb5`: E♭5
- `f#4`: F♯4

## Length Token
> I hate staccato because its duration is not the same as the size that it occupies

- a length token starts with `[0-9]+`, identifying it's note value
  - e.g. `4` is a 4th note, `16` is a 16th note 
- then it's followed by gibberish that I try to do my best to simplify as much as possible

### Dotted Note
> 1.5x duration and size

![](../assets/dotted.png)
<details><summary>input</summary>

```
====== BPM =======
    143
==================
    4. d#4 b4 d#5
    4. g#4 b4 g#5
    4  a#4 c#5 a#5

    4. d#4 b4 d#5
    4. g#4 b4 g#5
    4  a#4 c#5 a#5

    4  b4 d#5 b5
    4  g#4 b4 g#5
    4  a#4 c#5 a#5
    4  f#4 a#4 f#5

    1  g#4 b4 g#5
```
</details>

