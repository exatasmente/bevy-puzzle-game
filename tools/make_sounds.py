#!/usr/bin/env python3
"""Writes the game's sound effects to assets/sfx/.

The sounds are synthesised here rather than downloaded so the repository owns
them outright: no licence to track, nothing to re-source if a link rots, and the
whole set regenerates from this file. They are deliberately plain — short
tones with a soft envelope, closer to a watch beep than to a sound effect —
because the game is played in bursts and anything characterful becomes
irritating by the fiftieth round.

    python3 tools/make_sounds.py

Output is 16-bit mono WAV at 22.05 kHz. WAV rather than Ogg because it needs no
encoder: the whole script is standard library. Bevy's default features cover Ogg
only, so `Cargo.toml` enables `wav`.
"""

import math
import os
import struct
import wave

SAMPLE_RATE = 22050
OUTPUT = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "assets", "sfx")


def envelope(position, total, attack=0.01, release=0.35):
    """Fade in and out, so a tone starts and ends without a click."""
    seconds = position / SAMPLE_RATE
    length = total / SAMPLE_RATE
    remaining = length - seconds

    level = 1.0
    if seconds < attack:
        level *= seconds / attack
    if remaining < release:
        level *= max(0.0, remaining / release)
    return level


def tone(frequency, seconds, volume=0.35, harmonic=0.25):
    """A sine with a touch of its octave, which keeps it from sounding hollow."""
    total = int(SAMPLE_RATE * seconds)
    samples = []

    for position in range(total):
        time = position / SAMPLE_RATE
        value = math.sin(2 * math.pi * frequency * time)
        value += harmonic * math.sin(4 * math.pi * frequency * time)
        samples.append(value * volume * envelope(position, total))

    return samples


def sweep(start, end, seconds, volume=0.35):
    """A tone that slides in pitch — used where a single note reads as flat."""
    total = int(SAMPLE_RATE * seconds)
    samples = []
    phase = 0.0

    for position in range(total):
        share = position / max(1, total - 1)
        frequency = start + (end - start) * share
        phase += 2 * math.pi * frequency / SAMPLE_RATE
        samples.append(math.sin(phase) * volume * envelope(position, total))

    return samples


def join(*parts):
    """Plays parts one after another."""
    joined = []
    for part in parts:
        joined.extend(part)
    return joined


def write(name, samples):
    os.makedirs(OUTPUT, exist_ok=True)
    path = os.path.join(OUTPUT, name)

    with wave.open(path, "w") as handle:
        handle.setnchannels(1)
        handle.setsampwidth(2)
        handle.setframerate(SAMPLE_RATE)
        handle.writeframes(
            b"".join(
                struct.pack("<h", int(max(-1.0, min(1.0, sample)) * 32767))
                for sample in samples
            )
        )

    print("{} ({} bytes)".format(path, os.path.getsize(path)))


def main():
    # A hit: two notes up. Short, so a fast run does not turn into a drone.
    write("hit.wav", join(tone(660, 0.06), tone(990, 0.10)))

    # A miss: one note down, quieter than the hit. A miss should be legible,
    # not punishing — the screen shake already carries the bad news.
    write("miss.wav", sweep(320, 180, 0.16, volume=0.28))

    # A level up: an arpeggio, the only sound in the game allowed to be pleased
    # with itself.
    write("level.wav", join(tone(523, 0.07), tone(659, 0.07), tone(784, 0.16)))

    # A button: a click with no melody, so it never competes with the round.
    write("tap.wav", tone(440, 0.045, volume=0.22, harmonic=0.0))


if __name__ == "__main__":
    main()
