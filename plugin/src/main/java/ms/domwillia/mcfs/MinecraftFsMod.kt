package ms.domwillia.mcfs

import ms.domwillia.mcfs.ipc.IpcChannel
import net.fabricmc.api.ModInitializer
import org.apache.logging.log4j.LogManager
import java.io.IOException
import kotlin.io.path.exists

class MinecraftFsMod : ModInitializer {
    override fun onInitialize() = try {
        // close on shutdown
        Runtime.getRuntime().addShutdownHook(Thread {
            try {
                IPC?.close()
            } catch (e: IOException) {
                LOGGER.catching(e)
            }
        })

        val watchdog = Thread {
            val socketPath = IpcChannel.socketPath()
            while (true) {
                LOGGER.info("Initialising IPC")
                val thread = reinit()

                while (thread.isAlive && socketPath.exists()) {
                    Thread.sleep(1000)
                }

                IPC!!.close()
            }
        }
        watchdog.isDaemon = true
        watchdog.start()


    } catch (e: IOException) {
        LOGGER.catching(e)
        LOGGER.error("Failed to start")
    }

    companion object {
        val LOGGER = LogManager.getLogger("mcfs")!!
        var IPC: IpcChannel? = null

        fun reinit(): Thread {
            IPC = IpcChannel()

            // run command processing on a new thread
            val t = Thread(IPC)
            t.isDaemon = true
            t.start()

            return t
        }
    }
}