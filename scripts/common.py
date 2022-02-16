from __future__ import annotations

import re
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import ClassVar, Optional
from signal import signal, SIGPIPE, SIG_DFL


class IoException(Exception):
    def __init__(self, mc, path, exc):
        self.path = path.relative_to(mc.mnt)
        self.exc = exc
        self.message = f"Failed to access '{self.path}': {exc}"

    def __str__(self):
        return self.message


class Minecraft:
    def __init__(self, mnt: Path):
        if not mnt.is_dir():
            raise RuntimeError("Mount dir is invalid")

        # mild validation
        if not (mnt / "player").is_dir():
            raise RuntimeError("Mount dir missing player dir, sure it's a mount point?")

        # make it work nice with pipes such as `head`
        signal(SIGPIPE, SIG_DFL)

        self.mnt = mnt

    @classmethod
    def from_args(cls) -> Minecraft:
        args = sys.argv
        try:
            mnt_dir = Path(args[1])
        except IndexError:
            print("expected mnt directory as first arg")
            exit(1)

        try:
            return Minecraft(mnt_dir)
        except Exception as e:
            print(f"error: {e}")
            exit(1)

    # TODO provide world or default to players
    def iter_entities(self):
        world = self.mnt / "player" / "world"
        entities = world / "entities" / "by-id"

        for entity in entities.glob("*"):
            entity_id = int(entity.name)
            entity_ty = self._read(entity / "type")
            entity_pos = Position.from_string(self._read(entity / "position"))
            yield Entity(entity_id, entity_ty, entity_pos)

    def _read(self, p: Path) -> str:
        try:
            return p.read_text()
        except OSError as e:
            raise IoException(self, p, e)


@dataclass
class Position:
    x: float
    y: float
    z: float

    SPLIT_PATTERN: ClassVar[re.Pattern] = re.compile(r"\s|,")

    @classmethod
    def from_string(cls, s: str) -> Optional[Position]:
        [x, y, z] = cls.SPLIT_PATTERN.split(s, 2)
        try:
            return Position(float(x), float(y), float(z))
        except ValueError:
            raise RuntimeError(f"invalid position '{s}'")


@dataclass
class Entity:
    id: int
    type: str
    pos: Position

