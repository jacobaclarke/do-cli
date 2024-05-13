# DOIT

A simple task runners for the lazy.

## Installation

```bash
cargo install --git https://github.com/jacobaclarke/doit-cli.git
cargo install doit-cli
```

## Usage

- [Getting Started](#getting-started)
-   [Environment Variables](#environment-variables)
    -   [Global Environment Variables](#global-environment-variables)
    -   [Overriding Environment Variables from Shell](#overriding-environment-variables-from-shell)
    -   [Overriding Environment Variables from Task](#overriding-environment-variables-from-task)
-   [Subdirectories](#subdirectories)
    -   [Subdirectories Local](#subdirectories-local)
    -   [Nested `do.yaml` files](#nested-doyaml-files)

### Getting Started

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

#### Parallel Execution

```yaml
# do.yaml
tasks:
  hello:
    cmd:
      - echo "hello"
      - echo "world"
```


```bash
$ doit hello
hello
world
```

### Environment Variables

#### Global Environment Variables

```yaml
# do.yaml
env:
  NAME: world
tasks:
  hello:
    cmd: Hello $NAME
```
```bash
$ doit hello
Hello world
```

#### Overriding Environment Variables from Shell

```yaml
# do.yaml
env:
  NAME: jimmy
tasks:
  hello:
    cmd: Hello $NAME
```
```bash
$ NAME=world doit hello
Hello world
```

#### Overriding Environment Variables from Task

```yaml
# do.yaml
env:
  NAME: jimmy
tasks:
  hello:
    cmd: Hello $NAME
    env:
      NAME: world
```
```bash
$ doit hello
Hello world
```

## Subdirectories


```
├── parent
│   ├── child
├── do.yaml
```

```yaml
# /parent/do.yaml
task:
  pwd:
    cmd: pwd
```

```bash
$ cd child
$ doit pwd
/parent
```

### Subdirectories Local

```
├── parent
│   ├── child
├── do.yaml
```

```yaml
# /parent/do.yaml
task:
  pwd:
    cmd: pwd
    local: true
```

```bash
$ cd child
$ doit pwd
/parent/child
```


### Nested `do.yaml` files

```yaml
# /parent/do.yaml
task:
  hello:
    cmd: echo hello world
```

```yaml
# /parent/child/do.yaml
task:
  greet:
    cmd: doit hello
```

```bash
$ cd child
$ doit greet
hello world
```