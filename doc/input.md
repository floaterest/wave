# Input Format
> prepare to read some ugly regex

## Tokens
- tokens is defined as `(\S+)` (group of nonspace characters)
- the first token of the line is defined as `^\s*(\S+)` (first group of nonspace characters in line after left trimmed)
  - since each line gets trimmed, it is possible to pad any amount of spaces for alignment or indentiation

## Lines
> see each section for a better explaination of each token type

each line of the input file can be one of the 4 followings:
  - the line is both left and right trimmed before its identification
  - the identification is done in order


1. [BPM](#bpm)
2. [Repeat](#repeat)
3. [Chords](#chords)
4. [Comment](#comment)

## BPM
> a line that only contains one unsigned integer

- change current BPM
- there must be a BPM token before the first chords, otherwise the program doesn't know the length of the chord
- BPM can be changed midway

## Repeat
> don't repeat yourself (aka DRY code)

see [repeat.md](./repeat.md)


## Chords
> a line can contain multiple chords, a chord can contain multiple notes/captures<br>

### Notes
> me trying to implement [every music symbols](https://en.wikipedia.org/wiki/List_of_musical_symbols)

see [note.md](./note.md)

### Captures
> write even DRYer inputs

see [capture.md](./capture.md)

## Comments
> ignored by the program because it does not care

can be used to (for humans only)

- separate bars/staves/pages
- create foldable sections using different indents
