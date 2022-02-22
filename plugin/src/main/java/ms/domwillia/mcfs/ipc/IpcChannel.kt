package ms.domwillia.mcfs.ipc

import MCFS.GameRequest
import com.google.flatbuffers.FlatBufferBuilder
import ms.domwillia.mcfs.MinecraftFsMod
import java.io.IOException
import java.net.StandardProtocolFamily
import java.net.UnixDomainSocketAddress
import java.nio.ByteBuffer
import java.nio.ByteOrder
import java.nio.channels.ServerSocketChannel
import java.nio.channels.SocketChannel
import java.nio.file.Path
import java.nio.file.Paths
import java.util.concurrent.atomic.AtomicBoolean

class IpcChannel : Runnable {
    private val channel: ServerSocketChannel
    private val keepRunning = AtomicBoolean(true)

    @Throws(IOException::class)
    fun close() {
        channel.close()
        keepRunning.set(false)
    }

    @ExperimentalUnsignedTypes
    override fun run() {
        val lenBuf = ByteBuffer.wrap(ByteArray(4)).order(ByteOrder.LITTLE_ENDIAN)
        val buf = ByteBuffer.allocate(8192)
        val responseBuilder = FlatBufferBuilder(8192)
        val executor = Executor(responseBuilder)
        while (keepRunning.get()) {
            var client: SocketChannel? = null
            try {
                client = channel.accept()

                while (true) {
                    // read len
                    client.read(lenBuf.clear())
                    val len = lenBuf.flip().int
                    MinecraftFsMod.LOGGER.info("Reading $len bytes")

                    // read data
                    client.read(
                        buf.clear()
                            .order(ByteOrder.LITTLE_ENDIAN)
                            .limit(len)
                    )
                    buf.rewind()

                    // log bytes
                    // MinecraftFsMod.LOGGER.info(buf.array().copyOf(len).joinToString() { b -> "%02x".format(b) })

                    val request = GameRequest.getRootAsGameRequest(buf);
                    val response = executor.execute(request)

                    val responseSize = response.remaining()
                    buf.clear()
                        .order(ByteOrder.LITTLE_ENDIAN)
                        .putInt(responseSize)
                        .put(response)
                        .flip()

                    MinecraftFsMod.LOGGER.info("Writing $responseSize bytes")

                    // log bytes
                    // MinecraftFsMod.LOGGER.info(buf.array().copyOf(responseSize).joinToString() { b -> "%02x".format(b) })
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
        val path = socketPath();
        path.toFile().delete() // ensure we create it ourselves
        val address = UnixDomainSocketAddress.of(path)
        MinecraftFsMod.LOGGER.info("Binding to socket $address")

        channel = ServerSocketChannel.open(StandardProtocolFamily.UNIX)
        channel.bind(address)
    }

    companion object {
        fun socketPath(): Path {
            val username = System.getenv("USER")
            val tmpdir = System.getProperty("java.io.tmpdir")
            return Paths.get(tmpdir, String.format("minecraft-fuse-%s", username ?: "user"))
        }

    }
}