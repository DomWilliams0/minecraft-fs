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
import java.nio.file.Paths

class IpcChannel : Runnable {
    private val channel: ServerSocketChannel

    @Throws(IOException::class)
    fun close() {
        channel.close()
    }

    @ExperimentalUnsignedTypes
    override fun run() {
        val buf = ByteBuffer.allocate(8192)
        val responseBuilder = FlatBufferBuilder(8192)
        val executor = Executor(responseBuilder)
        while (true) {
            var client: SocketChannel? = null
            try {
                client = channel.accept()

                while (true) {
                    buf.order(ByteOrder.LITTLE_ENDIAN)
                    buf.clear()
                    client.read(buf)
                    buf.rewind()

                    val len = buf.int
                    MinecraftFsMod.LOGGER.info("Reading $len bytes")

                    val request = GameRequest.getRootAsGameRequest(buf);
                    val response = executor.execute(request)

                    response?.let {
                        val responseSize = it.remaining()
                        buf.clear();
                        buf.order(ByteOrder.LITTLE_ENDIAN)
                        buf.putInt(responseSize)
                        buf.put(it)
                        buf.flip()
                        MinecraftFsMod.LOGGER.info("Writing $responseSize bytes")
                        client.write(buf)
                    }
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