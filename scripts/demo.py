import sys

from common import Minecraft, IoException, BlockPos


def tp_all():
    player = mc.player()
    if player is None:
        return

    for e in mc.iter_entities():
        try:
            e.teleport(player.position)
            print(f"teleported {e.id}")
        except IoException as exc:
            print(f"failed to tp {e.id}: {exc}")
            raise exc


def kill_all_other_than_player():
    player = mc.player()
    if player is None:
        return

    for e in mc.iter_entities(living_filter=True):
        if e.id != player.id:
            try:
                e.kill()
            except IoException as exc:
                print(f"failed to kill {e.id}: {exc}")


def blocks():
    player = mc.player()
    if player is None:
        return

    pos = player.pos.to_block_pos()
    pos.y -= 1

    sz = 3
    for dx in range(-sz, sz):
        for dz in range(-sz, sz):
            block_pos = BlockPos(pos.x + dx, pos.y, pos.z + dz)
            block = mc.block(player.world, block_pos)
            block.block_type = "cobblestone"


if __name__ == '__main__':
    mc = Minecraft.from_args()

    what = sys.argv[2]
    if what == "tp-all":
        tp_all()
    elif what == "killall":
        kill_all_other_than_player(),
    elif what == "blocks":
        blocks(),
    else:
        print("unknown demo")
