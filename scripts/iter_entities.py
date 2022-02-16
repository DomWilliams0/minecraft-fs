from common import Minecraft

if __name__ == '__main__':
    mc = Minecraft.from_args()
    for e in mc.iter_entities():
        print(e)
