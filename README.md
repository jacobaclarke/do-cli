# DOIT

A simple task runners for the lazy.


## Example Usage

```yaml
# do.yaml
env:
  NAME: world
tasks:
  hello:
    cmd: echo $GREETING $NAME
    env:
      GREETING: Hello
  ```


```bash
$ doit hello
Hello world
```