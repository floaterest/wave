# Capture
> should be easier to understand than Regex captures

## Keys
> any group of alphabetic characters, i.e. matching `[a-zA-Z]+`

- all captures are stored in a HashMap, mapping a key (as string) to a queue (of chords)

## Tokens
there are 5 types of tokens, given a key `a`

- capture to `a`: `(a)`
- pop front of `a`: `<a>`
- get front of `a`: `[a]`
- rotate `a`: `|a|`
- clear `a`: `{a}`

# Usage
## Capture
> (plays and) captures a chord and stores to a queue<br>
> `()` because it's like captures from Regex

### Capture a Chord
> e.g. push back a C major triad to the queue named `C`

```
(C) 4 c4 e4 g4
```

### Capture a Sequence as a Queue
> e.g. push back the first 3 notes of E major scale named `E`
```
(E) 4 e4
(E) 4 f#4
(E) 4 g#4
```
now the queue is
```
(E) e4 -> f#4 -> g#4
```

### Capture to Mulitple Queues
> e.g. write the first 5 notes of F minor scale, but store
> - first 3 notes of F minor scale in `f`
> - first 3 notes of A♭ major scale in `Ab`

```
(f) 4 f4
(f) 4 g4
(Ab) (f) 4 ab4
(Ab) 4 bb4
(Ab) 4 c5
```
now the queue is
```
 (f) f4 -> g4 -> ab4
(Ab) ab4 -> bb4 -> c5
```
