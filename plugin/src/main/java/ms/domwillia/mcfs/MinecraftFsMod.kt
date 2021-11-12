package ms.domwillia.mcfs

import ms.domwillia.mcfs.ipc.IpcChannel
import net.fabricmc.api.ModInitializer
import org.apache.logging.log4j.LogManager
import java.io.IOException

class MinecraftFsMod : ModInitializer {
    override fun onInitialize() {
        try {
            val ipc = IpcChannel()

            // close on shutdown
            Runtime.getRuntime().addShutdownHook(Thread {
                try {
                    ipc.close()
                } catch (e: IOException) {
                    LOGGER.catching(e)
                }
            })

            // run command processing on a new thread
            val t = Thread(ipc)
            t.isDaemon = true
            t.start()

        } catch (e: IOException) {
            LOGGER.catching(e)
            LOGGER.error("Failed to start")
        }
    }

    companion object {
        @JvmField
        val LOGGER = LogManager.getLogger("mcfs")!!
    }
}