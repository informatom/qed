+++
title = "Interlude: Whisper"
description = "Interlude: transcripts for your episodes with whisper.cpp, on your own computer — installing it, a first test, a whole episode, and the result in q.e.d."
menu = "main"
weight = 54
date = 2026-10-28
authors = ["Claude", "Stefan Haslinger"]
tags = ["podcasting", "transcripts", "whisper", "linux"]
summary = "No q.e.d. code this time: how to make a WebVTT transcript of an episode with Whisper, on your own machine, without a cloud service. Written while setting it up on a laptop, step by step."
redirect_from = ["/blog/2026-10-28-interlude-whisper"]
+++

q.e.d. shows a transcript in the player, on the episode page and in the
feeds (#048). It doesn't write the transcript itself, and it shouldn't
send your audio to a cloud service either. OpenAI's speech recognition
model **Whisper** runs on an ordinary computer, and
[whisper.cpp](https://github.com/ggml-org/whisper.cpp) is a fast C++
version of it that writes WebVTT, the format q.e.d. reads.

This post is the installation, done on a 2021 laptop (Intel Core
i7-1185G7, 4 cores, 32 GB RAM, no graphics card of its own, Linux Mint
22). The commands are for Debian, Ubuntu and Mint. On macOS, Homebrew
has a ready package (`brew install whisper-cpp`); on Windows, the
project's releases have builds.

## What you need

- About **2 GB of disk** for the model, plus a little for the program.
- **RAM:** 3 GB free is plenty for the large model.
- **A processor from the last few years.** The newer, the faster:
  whisper.cpp uses the vector instructions modern CPUs have (AVX2,
  AVX-512). A graphics card speeds it up a lot, but isn't required.
- **Build tools:** git, a C++ compiler, make and cmake. On a desktop
  Linux most are there already. Only cmake was missing here:

```sh
sudo apt install git build-essential cmake
```

cmake reads the project's build description (`CMakeLists.txt`) and
writes the actual build instructions for your machine.

## 1. Get the source

```sh
mkdir -p ~/src
git clone https://github.com/ggml-org/whisper.cpp ~/src/whisper.cpp
cd ~/src/whisper.cpp
git checkout $(git describe --tags --abbrev=0)
```

The last line switches to the newest release instead of the
development branch: `git describe --tags --abbrev=0` prints the most
recent tag.

## 2. Build it

```sh
cmake -B build -DCMAKE_BUILD_TYPE=Release
cmake --build build -j 8
```

The first command checks your compiler and processor and sets up the
`build/` folder. `Release` means optimized for speed, and by default it
builds for *this* processor, using whatever it offers. The second
compiles; `-j 8` runs 8 jobs at once (use your number of threads).
This takes a few minutes.

Check that it runs:

```sh
./build/bin/whisper-cli --help | head -20
```

The help names the formats it reads: `flac, mp3, ogg, wav`. Your episode
files work as they are, without converting.

### With the graphics chip: Vulkan (recommended)

Without further options, whisper.cpp computes on the processor only.
Its **Vulkan** backend uses the graphics chip as well: the Intel Iris
Xe built into many laptops, the Radeon graphics of AMD Ryzen mini PCs,
or a graphics card. Even the modest Iris Xe made it almost four times
faster (see the measurements below), so this is worth the extra step.

First check that Vulkan sees the chip (`vulkan-tools` has
`vulkaninfo`):

```sh
vulkaninfo --summary | grep -E "deviceName|deviceType"
```

The laptop shows `Intel(R) Iris(R) Xe Graphics (TGL GT2)` as an
`INTEGRATED_GPU`. A second entry, `llvmpipe`, is a software stand-in on
the processor; ignore it.

The build needs three more packages: the Vulkan headers, `glslc`, which
compiles the *shaders* (the small programs that run on the graphics
chip), and the SPIR-V headers, the format shaders compile to. Build into
a folder of its own, so the processor-only build stays for comparison:

```sh
sudo apt install libvulkan-dev glslc spirv-headers
cmake -B build-vulkan -DCMAKE_BUILD_TYPE=Release -DGGML_VULKAN=1
cmake --build build-vulkan -j 8
```

Without `spirv-headers`, cmake stops with *Could not find a package
configuration file provided by "SPIRV-Headers"*. Install it, delete the
half-configured folder (`rm -rf build-vulkan`, since cmake caches what
it found) and run both commands again. A note about a missing
`glslangValidator` doesn't matter: `glslc` is enough.

Started from `build-vulkan/bin/whisper-cli`, the output names the chip:

```text
ggml_vulkan: 0 = Intel(R) Iris(R) Xe Graphics (TGL GT2) (Intel open-source Mesa driver) | …
whisper_backend_init_gpu: using Vulkan0 backend
```

## 3. Download a model

```sh
sh ./models/download-ggml-model.sh large-v3-turbo
```

The **model** is the trained neural network, everything Whisper learned
about speech. The program is small and just runs it. The script fetches
`ggml-large-v3-turbo.bin` (1.6 GB) from Hugging Face into `models/`.
"turbo" is a slimmed-down large model: almost as accurate, much faster.
For German, the large models are clearly better than the small ones.

There are also **quantized** versions, the same model with its numbers
stored more compactly, e.g. `large-v3-turbo-q5_0` at about a third of
the size. On a processor they are often faster, with hardly any loss in
quality. Worth a try:

```sh
sh ./models/download-ggml-model.sh large-v3-turbo-q5_0
```

## 4. A first test: five minutes

```sh
time ./build/bin/whisper-cli \
  -m models/ggml-large-v3-turbo.bin \
  -l de \
  -d 300000 \
  -pp \
  -ovtt -of /tmp/whisper-test \
  -f ~/path/to/episode.mp3
```

- `time` measures the whole run; the `real` line at the end is what
  counts.
- `-m` is the model, `-l de` the language. **Always set the language**:
  the default is English, and German audio then comes out as an attempt
  at English.
- `-d 300000` processes only the first 300,000 milliseconds, five
  minutes.
- `-pp` shows progress, `-ovtt -of /tmp/whisper-test` writes
  `/tmp/whisper-test.vtt`.

Before it starts, whisper.cpp prints what it found. The line to look at
begins with `system_info`:

```text
system_info: n_threads = 4 / 8 | … AVX2 = 1 | … AVX512 = 1 | AVX512_VNNI = 1 | …
```

`= 1` means it uses those instructions. `n_threads = 4 / 8` is the
number of threads it computes with, out of the number available. Then
the result:

```sh
head -30 /tmp/whisper-test.vtt
```

```text
WEBVTT

00:00:01.000 --> 00:00:11.800
 Drei Schweinehunde. Von Null zum Halbmarathon. Der Podcast über Laufen, Gadgets und Motivation.

00:00:13.640 --> 00:00:19.500
 Stefan aus dem Maschinenraum der Postproduktion hier. Diese Episode besteht aus drei Teilen.
```

That's 3 Schweinehunde, episode 51, as `large-v3-turbo` heard it:
names, technical words and punctuation came out right. Now and then a
word is wrong. "gegen sechs aufstehen" became "gegen Sex aufstehen", so
read the transcript once before you publish it.

From the test's time you can estimate a whole episode: an episode 18
times as long takes about 18 times as long.

### Measured on the laptop

The same five minutes of 3 Schweinehunde, once on the processor, once
with Vulkan on the Iris Xe. At the end, whisper.cpp lists where the
time went (`whisper_print_timings`):

| | Processor (`build`) | Iris Xe (`build-vulkan`) |
|---|---|---|
| *encode*: the network listens to each 30-second window | 342 s | 74 s |
| writing the text (*batchd*, *sample*, *prompt*) | 54 s | 26 s |
| total (`real`) | 6 min 48 s | 1 min 51 s |
| per hour of audio | about 82 min | about 22 min |
| processor busy (`user`) | 27 min, all 4 cores | under 1 min |

The encoder is most of the work, and it's the kind of arithmetic
graphics chips are made for. With Vulkan, an 88-minute episode takes
about half an hour, and the processor stays free for other work. The
text was as good either way: the Vulkan run cut it into shorter cues
and got a few passages better and a few worse.

## 5. Faster, if needed

The biggest step is the graphics chip (Vulkan, above). Beyond that, try
these one at a time on the same five minutes:

- **A quantized model:** `-m models/ggml-large-v3-turbo-q5_0.bin` or
  `q8_0`. Less data to move for the encoder, which does most of the
  work.
- **Threads:** `-t 4` or `-t 8`, for the processor-only build. More
  threads than real cores often don't help; sometimes they do.
- **Simpler decoding:** `-bs 1`. By default Whisper follows five
  candidate texts at once and keeps the best ("beam search"). With one,
  writing the text is faster, with slightly more mistakes. Since that's
  the smaller part of the work, it gains little.

Or let it run overnight. Speed only matters if you're waiting for it.

## 6. Install it, with a small script

So that `whisper-cli` works from any folder, link it into `~/.local/bin`,
which most Linux systems have on the `PATH`. Take the Vulkan build if
you made one (otherwise `build` instead of `build-vulkan`):

```sh
mkdir -p ~/.local/bin
ln -sf ~/src/whisper.cpp/build-vulkan/bin/whisper-cli ~/.local/bin/whisper-cli
```

`-f` replaces a link that's already there.

And a script that saves typing, `~/.local/bin/transcribe`:

```sh
#!/bin/sh
# transcribe episode.mp3 [more.mp3 …]: writes episode.vtt next to each
# file. The language is German unless WHISPER_LANG says otherwise.
set -e
model="$HOME/src/whisper.cpp/models/ggml-large-v3-turbo.bin"
for file in "$@"; do
  whisper-cli -m "$model" -l "${WHISPER_LANG:-de}" -pp \
    -ovtt -of "${file%.*}" -f "$file"
done
```

```sh
chmod +x ~/.local/bin/transcribe
transcribe 3sh051_trilogie.mp3
```

`${file%.*}` is the file name without its extension, so
`3sh051_trilogie.mp3` gets `3sh051_trilogie.vtt` next to it.

To update whisper.cpp later: `git fetch --tags`, check out the newest
tag as in step 1, and build again as in step 2 (both builds, if you
have both).

## 7. Into q.e.d.

1. **Transcripts** tab: *Import transcripts…* and pick the `.vtt` file.
2. In the episode's settings, at *Transcript*: *Choose…* and pick it.
3. Build or preview. The player has a transcript tab now, the feeds
   name the file, and if the episode layout has a **Transcript** block,
   the text is on the page too, folded away.

Whisper doesn't know who is speaking, so the transcript has no names.
It's still worth it: listeners can read along and jump to a passage,
and search engines read every word. q.e.d. splits transcripts without
speakers into paragraphs at the pauses (after this interlude, in the
next version). If you want names, add WebVTT voice tags by hand, at
least where the speaker changes:

```text
00:00:56.320 --> 00:00:57.880
<v Birgit Wimmer>Hallo, Grüße aus Regensburg.
```

If the name matches a contributor, the player shows their photo next to
the text.
