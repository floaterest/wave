# Repeat
> repeat the chords, not yourself

## Tokens
there are 4 tyes of repeat tokens

- RepeatStart `|:`
- RepeatEnd `:|`
- VoltaStart `|1.3.` (start volta 1 and 3)
- VoltaEnd `|`

# Usage
## No Voltas
> just do the same lines twice

e.g. to produce `A B A B`
```
|:
    A
    B
:|
```

## With Voltas
### Prevolta Only
> where voltas happen at the end of the repeat section

e.g. to produce `A B A C A B A C`
- traditional method:
  ```
  |:
      A
  |1.3.
      B
  :| |2.
      C
  :| |4.
      C
  |
  
  ```
- optimised method:
  ```
  |:
      A
  |1.3.
      B
  |2.4.
      C
  :|
  ```

### Postvolta only
> where voltas happen at the start of the repeat section

e.g. to produce `A C B C D`
- traditional method:
  ```
  A
  
  |:
      C
  |1.
      B
  :| |2.
      D
  |    
  ```
- optimised method:
  ```
  |: |1.
      A
  |2.
      B
  |
      C
  :|

  D
  ```


### Prevolta & Postvolta
> where voltas happen in the middle of the repeat section

e.g. to produce `A B D A C D`
- traditional method (using prevolta only):
  ```
  |:
      A
  |1.
      B
      D
  :| |2.
      C
      D
  |
  ```
- optimised method:
  ```
  |:
      A
  |1.
      B
  |2.
      C
  |
      D
  :|            
  ```
