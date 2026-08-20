#!/usr/bin/env python3
"""Simple extractor for the *musician-soul* repo.

It walks the ``src`` directory, finds any audio‑related files (by extension),
and returns a list of canonical strings that can be fed to the embedder.

The current placeholder implementation treats every ``.wav``, ``.mp3``,
``.flac`` and ``.ogg`` file as a separate *audio prompt* and returns a
human‑readable description:

    "Audio file <relative_path> – size <bytes> bytes"

In a real system you would extract metadata (sample rate, duration, tags),
but this scaffold is enough to demonstrate the end‑to‑end pipeline.
"""
import os
from pathlib import Path

AUDIO_EXTS = {".wav", ".mp3", ".flac", ".ogg"}


def extract_audio_prompts(repo_root: str = None):
    """Return a list of textual prompts for all audio files.

    Args:
        repo_root: Path to the ``musician-soul`` repository. If ``None`` the
            function uses the directory of this file's parent.
    Returns:
        List[str]: Human‑readable descriptions of each audio asset.
    """
    if repo_root is None:
        repo_root = Path(__file__).resolve().parent
    src_dir = Path(repo_root) / "src"
    prompts = []
    for root, _, files in os.walk(src_dir):
        for f in files:
            ext = Path(f).suffix.lower()
            if ext in AUDIO_EXTS:
                full_path = Path(root) / f
                size = full_path.stat().st_size
                rel = full_path.relative_to(repo_root)
                prompts.append(f"Audio file {rel} – size {size} bytes")
    return prompts


if __name__ == "__main__":
    for p in extract_audio_prompts():
        print(p)
