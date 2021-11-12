package ms.domwillia.mcfs.ipc

import com.google.gson.JsonElement
import com.google.gson.JsonNull
import java.lang.Runnable
import java.io.IOException
import ms.domwillia.mcfs.MinecraftFsMod
import net.minecraft.util.JsonHelper
import java.lang.Exception
import java.nio.file.Paths
import java.net.UnixDomainSocketAddress
import java.net.StandardProtocolFamily
import java.nio.ByteBuffer
import java.nio.channels.ServerSocketChannel
import java.nio.channels.SocketChannel
import java.nio.charset.StandardCharsets

class IpcChannel : Runnable {
    private val channel: ServerSocketChannel

    @Throws(IOException::class)
    fun close() {
        channel.close()
    }

    override fun run() {
        val buf = ByteBuffer.allocate(8192)
        while (true) {
            var client: SocketChannel? = null
            try {
                client = channel.accept()

                while (true) {
                    buf.clear()
                    client.read(buf)
                    buf.rewind()

                    val len = buf.int
                    MinecraftFsMod.LOGGER.info("Reading $len bytes")

                    val json = ByteArray(len)
                    buf[json, 0, len]
                    val obj = JsonHelper.deserialize(String(json))

                    val response: JsonElement = try {
                        CommandExecutor.execute(obj["ty"].asString)
                    } catch (e: Exception) {
                        // TODO send back response in error json for specific errors
                        MinecraftFsMod.LOGGER.catching(e)
                        JsonNull.INSTANCE
                    }
                    buf.clear()

                    val respBytes = response.toString().toByteArray(StandardCharsets.UTF_8)
                    buf.putInt(respBytes.size)
                    buf.put(respBytes)
                    buf.flip()
                    MinecraftFsMod.LOGGER.info("Writing ${respBytes.size} bytes")
                    client.write(buf)
                }
            } catch (e: Exception) {
                MinecraftFsMod.LOGGER.catching(e)
                if (client != null) {
                    try {
                        client.close()
                    } catch (e2: IOException) {
                        MinecraftFsMod.LOGGER.catching(e2)
                        break
                    }
                }
            }
        }
    }

    init {
        val username = System.getenv("USER")
        val tmpdir = System.getProperty("java.io.tmpdir")
        val path = Paths.get(tmpdir, String.format("minecraft-fuse-%s", username ?: "user"))
        path.toFile().delete() // ensure we create it ourselves
        val address = UnixDomainSocketAddress.of(path)
        MinecraftFsMod.LOGGER.info("Binding to socket $address")

        channel = ServerSocketChannel.open(StandardProtocolFamily.UNIX)
        channel.bind(address)
    }
}