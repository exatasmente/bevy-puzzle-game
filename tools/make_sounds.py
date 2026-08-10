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


# ---------------------------------------------------------------------------
# Music
#
# Two loops: one for the menu, one for a round in progress. They are built from
# a bar grid so the end meets the beginning exactly, and any note still ringing
# at the end is wrapped around to the start — a loop with a decaying tail cut
# off at the seam clicks once per repeat, which is the fastest way to make a
# player reach for the mute button.
#
# Both are deliberately restrained. This is a concentration game played in
# bursts; the music is there so silence does not read as a broken build, not to
# be listened to. The lead in the menu theme does not appear in the round track
# at all, because a melody is exactly the thing that pulls attention off a
# board you are trying to compare colours on.
# ---------------------------------------------------------------------------

# Equal temperament from A4. Note names are only ever read by this file.
NOTES = {"C": -9, "C#": -8, "D": -7, "D#": -6, "E": -5, "F": -4,
         "F#": -3, "G": -2, "G#": -1, "A": 0, "A#": 1, "B": 2}


def pitch(name):
    """"A3" -> 220.0, "C#5" -> 554.4."""
    step = NOTES[name[:-1]]
    octave = int(name[-1])
    return 440.0 * (2.0 ** (step / 12.0 + (octave - 4)))


def adsr(position, total, attack, decay, sustain, release):
    """Attack, decay, sustain, release, in seconds and a level."""
    seconds = position / SAMPLE_RATE
    length = total / SAMPLE_RATE

    if seconds < attack:
        return seconds / attack if attack else 1.0
    if seconds < attack + decay:
        share = (seconds - attack) / decay if decay else 1.0
        return 1.0 + (sustain - 1.0) * share

    remaining = length - seconds
    if remaining < release:
        return sustain * max(0.0, remaining / release)
    return sustain


def voice(frequency, seconds, volume, partials, envelope):
    """Additive synthesis: a list of (multiple, level) over one envelope.

    Everything in these tracks is one of three shapes — a pad, a bass and a
    pluck — and they differ only in their partials and their envelope, so one
    function covers all of them.
    """
    total = int(SAMPLE_RATE * seconds)
    samples = [0.0] * total

    for multiple, level in partials:
        angular = 2.0 * math.pi * frequency * multiple / SAMPLE_RATE
        for position in range(total):
            samples[position] += level * math.sin(angular * position)

    return [sample * volume * envelope(position, total)
            for position, sample in enumerate(samples)]


def pad(name, seconds, volume=0.16):
    """Slow swell. Carries the harmony without ever arriving."""
    return voice(
        pitch(name), seconds, volume,
        [(1, 1.0), (2, 0.30), (3, 0.12), (4, 0.06)],
        lambda p, t: adsr(p, t, 0.35, 0.30, 0.75, 0.60),
    )


def bass(name, seconds, volume=0.30):
    """Round and short, so the pulse is felt rather than heard."""
    return voice(
        pitch(name), seconds, volume,
        [(1, 1.0), (2, 0.22)],
        lambda p, t: adsr(p, t, 0.01, 0.10, 0.55, 0.14),
    )


def pluck(name, seconds, volume=0.18):
    """Odd harmonics and a fast decay: the closest this gets to a lead."""
    return voice(
        pitch(name), seconds, volume,
        [(1, 1.0), (3, 0.28), (5, 0.12)],
        lambda p, t: adsr(p, t, 0.005, 0.22, 0.35, 0.30),
    )


class Track:
    """A fixed-length buffer notes are dropped into at a time in seconds.

    Writing wraps around the end, which is what makes the loop seamless: the
    tail of the last chord becomes the sound already ringing when the loop
    starts again.
    """

    def __init__(self, seconds):
        self.length = int(SAMPLE_RATE * seconds)
        self.samples = [0.0] * self.length

    def add(self, at, samples):
        start = int(SAMPLE_RATE * at)
        for offset, sample in enumerate(samples):
            self.samples[(start + offset) % self.length] += sample

    def normalised(self, peak):
        loudest = max(abs(sample) for sample in self.samples) or 1.0
        scale = peak / loudest
        return [sample * scale for sample in self.samples]


def theme():
    """The menu. Four bars of Am - F - C - G at 80bpm, so twelve seconds."""
    beat = 60.0 / 80.0
    bar = beat * 4
    track = Track(bar * 4)

    chords = [
        ("A3", ["A3", "C4", "E4"], ["A2", "E3"]),
        ("F3", ["F3", "A3", "C4"], ["F2", "C3"]),
        ("C4", ["C4", "E4", "G4"], ["C3", "G3"]),
        ("G3", ["G3", "B3", "D4"], ["G2", "D3"]),
    ]

    # A phrase per bar, pentatonic, leaving the fourth bar mostly open so the
    # loop has somewhere to breathe before it comes round again.
    melody = [
        [(0.0, "E5", 1.5), (1.5, "C5", 0.75), (2.25, "D5", 1.0)],
        [(0.5, "C5", 1.0), (1.5, "A4", 1.5)],
        [(0.0, "G4", 0.75), (0.75, "C5", 0.75), (1.5, "E5", 1.75)],
        [(1.0, "D5", 0.75), (1.75, "B4", 1.5)],
    ]

    for index, (_, notes, walk) in enumerate(chords):
        at = bar * index

        for note in notes:
            track.add(at, pad(note, bar * 1.05))

        # Two bass notes a bar: root, then the fifth on the third beat.
        track.add(at, bass(walk[0], beat * 1.6))
        track.add(at + beat * 2, bass(walk[1], beat * 1.6))

        for offset, note, length in melody[index]:
            track.add(at + offset * beat, pluck(note, beat * length))

    return track.normalised(0.72)


def round_music():
    """A round in progress. Eight bars at 96bpm, so twenty seconds.

    No melody: a pulse and a slow harmony. The player is comparing colours, and
    a tune with a shape to follow competes for exactly the attention the board
    needs. What this has to do is fill the silence and mark time.
    """
    beat = 60.0 / 96.0
    bar = beat * 4
    track = Track(bar * 8)

    progression = [
        (["A3", "C4", "E4"], "A2"),
        (["A3", "C4", "E4"], "A2"),
        (["F3", "A3", "C4"], "F2"),
        (["F3", "A3", "C4"], "F2"),
        (["C4", "E4", "G4"], "C3"),
        (["C4", "E4", "G4"], "C3"),
        (["G3", "B3", "D4"], "G2"),
        (["E3", "G#3", "B3"], "E2"),
    ]

    for index, (notes, root) in enumerate(progression):
        at = bar * index

        for note in notes:
            track.add(at, pad(note, bar * 1.05, volume=0.13))

        # A note on every beat. Quiet, and low enough to sit under everything.
        for step in range(4):
            emphasis = 0.30 if step == 0 else 0.17
            track.add(at + beat * step, bass(root, beat * 0.9, volume=emphasis))

        # One arpeggio note a bar, alternating high and low, so the loop moves
        # without ever proposing a tune.
        note = notes[2] if index % 2 == 0 else notes[1]
        track.add(at + beat * 2.5, pluck(note, beat * 1.2, volume=0.09))

    return track.normalised(0.60)


def main():
    # A hit: two notes up. Short, so a fast run does not turn into a drone.
    write("hit.wav", join(tone(660, 0.06, volume=0.55), tone(990, 0.10, volume=0.55)))

    # A miss: one note down, quieter than the hit. A miss should be legible,
    # not punishing — the screen shake already carries the bad news.
    write("miss.wav", sweep(320, 180, 0.16, volume=0.44))

    # A level up: an arpeggio, the only sound in the game allowed to be pleased
    # with itself.
    write("level.wav", join(
        tone(523, 0.07, volume=0.55),
        tone(659, 0.07, volume=0.55),
        tone(784, 0.16, volume=0.55),
    ))

    # A button: a click with no melody, so it never competes with the round.
    write("tap.wav", tone(440, 0.045, volume=0.30, harmonic=0.0))

    write("theme.wav", theme())
    write("round.wav", round_music())


if __name__ == "__main__":
    main()
