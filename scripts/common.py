from __future__ import annotations

import re
import sys
from dataclasses import dataclass, field
from pathlib import Path
from signal import signal, SIGPIPE, SIG_DFL
from typing import ClassVar, Optional


class IoException(Exception):
    def __init__(self, mc, path, exc, op):
        self.path = path.relative_to(mc.mnt)
        self.exc = exc
        self.message = f"Failed to {op} '{self.path}': {exc}"

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
    def iter_entities(self, living_filter=None, world=None):
        if world is not None:
            world_name = world
            world = self.mnt / "worlds" / world
        else:
            world = self.mnt / "player" / "world"
            world_name = world.readlink().name
        entities = world / "entities" / "by-id"
        use_living_filter = isinstance(living_filter, bool)

        for entity in entities.glob("*"):
            alive = self._exists(entity / "alive")
            if not alive:
                continue

            living = self._exists(entity / "living")
            if use_living_filter and living != living_filter:
                continue

            # TODO lazily evaluate fields? needs to remember its world
            entity_id = int(entity.name)
            entity_ty = self._read(entity / "type")
            entity_pos = Position.from_string(self._read(entity / "position"))
            yield Entity(entity_id, entity_ty, entity_pos, world_name, mc=self)

    def _read(self, p: Path) -> str:
        try:
            return p.read_text()
        except OSError as e:
            raise IoException(self, p, e, "check existence of")

    def _write(self, p: Path, text: str):
        try:
            p.write_text(text)
        except OSError as e:
            raise IoException(self, p, e, "write")

    def _exists(self, p: Path) -> bool:
        try:
            return p.exists()
        except OSError as e:
            raise IoException(self, p, e, "read")


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

    def __repr__(self):
        return f"{self.x},{self.y},{self.z}"


@dataclass
class Entity:
    id: int
    type: str
    pos: Position
    world: str
    mc: Minecraft = field(repr=False)

    def teleport(self, target: Position):
        """
        Position is treated as if it's in the same world
        """
        path = self.mc.mnt / "worlds" / self.world / "entities" / "by-id" / str(self.id) / "position"
        self.mc._write(path, repr(target))
