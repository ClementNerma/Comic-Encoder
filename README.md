# Comic Encoder

Comic Encoder is a command-line tool that enables compilation and extraction of comic archives.

## Features

Main features are:

* Compiling groups of chapters into several volumes (e.g. you have dozens of chapters and want to make volumes of 10 chapters)
* Compiling chapters into individual volumes (e.g. you want one volume per chapter, or you simply want to compile multiple comics at once)
* Compiling groups of chapters into a single volume (e.g. you have all chapters of a book and want to get a single archive out of it)

## Usage

Considering the following directory at `/home/me/book`:

```
/home/me/book
├── FirstChapter_1
├── MyChapter_10
├── MyChapter_11
├── MyChapter_2
├── MyChapter_3
├── MyChapter_4
├── MyChapter_5
├── MyChapter_6
├── MyChapter_7
├── MyChapter_8
├── MyChapter_9
└── ZChapter_12
```

### Compile multiple chapters into volumes of 5 chapters each

```
> cargo run --release -- volumify --compile 5 -i /home/me/book -o ./build/

[ 0m  0.000s] INFO: Going to compile chapters 1 to 12 (12 out of 12, 0 were ignored) into 3 volumes.
[ 0m  0.004s] INFO: Successfully created volume 1 (chapters 01 to 05) in 'Volume-1.cbz', containing 0 pages.
[ 0m  0.008s] INFO: Successfully created volume 2 (chapters 06 to 10) in 'Volume-2.cbz', containing 0 pages.
[ 0m  0.010s] INFO: Successfully created volume 3 (chapters 11 to 12) in 'Volume-3.cbz', containing 0 pages.
[ 0m  0.011s] INFO: Done in 0m 0.011s.

build
├── Volume-1.cbz
├── Volume-2.cbz
└── Volume-3.cbz
```

### Compile chapters into individual volumes

```shell
> cargo run --release -- volumify --individual -i /home/me/book -o ./build/

[ 0m  0.000s] INFO: Going to compile chapters 1 to 12 (12 out of 12, 0 were ignored) into 12 volumes.
[ 0m  0.002s] INFO: Successfully created volume file 'FirstChapter_1.cbz', containing 0 pages.
[ 0m  0.004s] INFO: Successfully created volume file 'MyChapter_2.cbz', containing 0 pages.
[ 0m  0.006s] INFO: Successfully created volume file 'MyChapter_3.cbz', containing 0 pages.
[ 0m  0.008s] INFO: Successfully created volume file 'MyChapter_4.cbz', containing 0 pages.
[ 0m  0.011s] INFO: Successfully created volume file 'MyChapter_5.cbz', containing 0 pages.
[ 0m  0.013s] INFO: Successfully created volume file 'MyChapter_6.cbz', containing 0 pages.
[ 0m  0.015s] INFO: Successfully created volume file 'MyChapter_7.cbz', containing 0 pages.
[ 0m  0.017s] INFO: Successfully created volume file 'MyChapter_8.cbz', containing 0 pages.
[ 0m  0.018s] INFO: Successfully created volume file 'MyChapter_9.cbz', containing 0 pages.
[ 0m  0.020s] INFO: Successfully created volume file 'MyChapter_10.cbz', containing 0 pages.
[ 0m  0.022s] INFO: Successfully created volume file 'MyChapter_11.cbz', containing 0 pages.
[ 0m  0.024s] INFO: Successfully created volume file 'ZChapter_12.cbz', containing 0 pages.
[ 0m  0.024s] INFO: Done in 0m 0.025s.

build
├── FirstChapter_1.cbz
├── MyChapter_10.cbz
├── MyChapter_11.cbz
├── MyChapter_2.cbz
├── MyChapter_3.cbz
├── MyChapter_4.cbz
├── MyChapter_5.cbz
├── MyChapter_6.cbz
├── MyChapter_7.cbz
├── MyChapter_8.cbz
├── MyChapter_9.cbz
└── ZChapter_12.cbz
```

### Compile multiple chapters into a single volume

```shell
cargo run --release -- volumify --single -i /home/me/book -o Book.cbz

[ 0m  0.000s] INFO: Going to compile chapters 1 to 12 (12 out of 12, 0 were ignored) into 1 volume.
[ 0m  0.004s] INFO: Successfully created volume 'Book.cbz' (chapters 01 to 12) in 'Book.cbz', containing 0 pages.
[ 0m  0.005s] INFO: Done in 0m 0.005s.
```

### Options

* `--start-chapter <num>`: ignore every chapter before the provided one (numbers start at 1)
* `--end-chapter <num>`: ignore every chapter after the provided one (numbers start at 1)
* `--dirs-prefix <prefix>`: only consider chapter directories that start by the provided prefix
* `--create-output-dir`: create the output directory if it does not exist yet
* `--extended-images-formats`: allow images with exotic formats, that may not be suppored by the vast majority of comics readers
* `--compress-losslessly`: compress all pictures losslessy - takes quite a bit of time, mostly useless on JPEG/PNG, but useful on BMP images
* `--silent`: do not display anything, except error messages

You can see additional parameters by calling `cargo run --release -- volumify --help`.

## Installation

Simply clone the project and run `cargo run --release` inside it, or `cargo build --release` to get a standalone executable.
