# Input Format
## General
  
each line of the input file can be any of the following (after being trimmed):

|           Description            | Regex Representation |     How the Program Will Interpret It     |       Example        |
| :------------------------------: | :------------------: | :---------------------------------------: | :------------------: |
|         empty/whitespace         |       `^\s*$`        |                  ignore                   |         `\t`         |
| starts with non-digit characters |        `^\D`         | ignore (treated as [comments](#comments)) | `==== anything ====` |
|    contains only one integer     |      `^(\d+)$`       |            set new BPM to `$1`            |        `200`         |
|     `<chord> [<chord> ...]`      |      see below       |                 see below                 |      see below       |

## Comments
> any line that doen't start with an ascii digit character

comments can be used for:

- seperators between 
    - [beamed notes](https://en.wikipedia.org/wiki/Beam_(music))
    - [bars/measures](https://en.wikipedia.org/wiki/Bar_(music))
    - [staves](https://en.wikipedia.org/wiki/Staff_(music))
    - pages
- indented folding

<details><summary>Example: use comments to separate bars and beams with indentation</summary>

![](../assets/comment.png)
```
==== BPM  =====
164
==== Bar 1 ====
	8 b2 f#4 b4
	8 f#5
	-----------
	8 e5
	8 f#5
	-----------
	8 d5
	8 a5
	-----------
	8 e5
	8 c#6
==== Bar 2 ====
	8 f#5
	8 d6
	-----------
	8 e5
	8 c#6
	-----------
	8 d5
	8 b5
	-----------
	8 c#5
	8 a5
==== Bar 3 ====
	8 f#4 b4 d5
	8 f#5
	-----------
	8 e5
	8 f#5
	-----------
	8 d5
	8 a5
	-----------
	8 c#6
	8 a5
==== Bar 4 ====
	8 d5
	8 f#5
	-----------
	8 c#5
	8 e5
	-----------
	8 b4
	8 d5
	-----------
	8 a4
	8 c#5
```
<details>

<!-- <details><summary></summary><details> -->